use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;

use chrono::Utc;

use crate::modules::audit_trail_log::domain::entity::AuditTrailLog;
use crate::modules::user_setting::application::dto::UpsertUserSettingRequest;
use crate::modules::user_setting::application::service::UserSettingService;
use crate::modules::user_setting::domain::{
    UserSetting, UserSettingDomainError, UserSettingRepository,
};
use crate::shared::cache::{CacheRepository, RedisCacheRepository};
use crate::shared::context::current_request_context;
use crate::shared::contracts::AuditTrailRecorder;
use crate::shared::errors::AppError;

const ENTITY_TYPE: &str = "user_setting";

/// Fires an audit trail write in the background so a slow/unavailable audit
/// sink never blocks or fails the actual setting mutation. `user_id` is both
/// the subject and the actor here -- every method in this module is
/// self-service, scoped to the caller's own `Claims::sub`.
fn spawn_audit_log(
    audit: Arc<dyn AuditTrailRecorder>,
    user_id: i32,
    action: &'static str,
    key: &str,
    old_values: Option<&UserSetting>,
    new_values: Option<&UserSetting>,
) {
    let ctx = current_request_context();
    let log = AuditTrailLog {
        id: 0,
        user_id: Some(user_id),
        action: action.to_string(),
        entity_type: ENTITY_TYPE.to_string(),
        entity_id: Some(key.to_string()),
        old_values: old_values.and_then(|s| serde_json::to_value(s).ok()),
        new_values: new_values.and_then(|s| serde_json::to_value(s).ok()),
        ip_address: ctx.ip_address,
        user_agent: ctx.user_agent,
        description: Some(format!("user {user_id} setting key {key}")),
        created_at: Utc::now(),
    };

    let key = key.to_string();
    tokio::spawn(async move {
        if let Err(err) = audit.record_audit_trail_log(log).await {
            tracing::error!(error = ?err, user_id, key, action, "failed to record user setting audit trail log");
        }
    });
}

const CACHE_TTL: Duration = Duration::from_secs(300);

fn list_cache_key(user_id: i32) -> String {
    format!("user_setting:list:{user_id}")
}

fn item_cache_key(user_id: i32, key: &str) -> String {
    format!("user_setting:item:{user_id}:{key}")
}

pub struct UserSettingServiceImpl {
    audit: Arc<dyn AuditTrailRecorder>,
    repo: Arc<dyn UserSettingRepository>,
    cache: Arc<RedisCacheRepository>,
}

impl UserSettingServiceImpl {
    pub fn new(
        audit: Arc<dyn AuditTrailRecorder>,
        repo: Arc<dyn UserSettingRepository>,
        cache: Arc<RedisCacheRepository>,
    ) -> Self {
        Self { audit, repo, cache }
    }

    async fn invalidate(&self, user_id: i32, key: &str) -> Result<(), AppError> {
        self.cache.delete(&item_cache_key(user_id, key)).await?;
        self.cache.delete(&list_cache_key(user_id)).await?;
        Ok(())
    }
}

#[async_trait]
impl UserSettingService for UserSettingServiceImpl {
    async fn list(&self, user_id: i32) -> Result<Vec<UserSetting>, AppError> {
        let ck = list_cache_key(user_id);
        if let Some(cached) = self.cache.get::<Vec<UserSetting>>(&ck).await? {
            return Ok(cached);
        }

        let settings = self.repo.list_for_user(user_id).await?;
        self.cache.set(&ck, &settings, CACHE_TTL).await?;
        Ok(settings)
    }

    async fn get(&self, user_id: i32, key: &str) -> Result<UserSetting, AppError> {
        let ck = item_cache_key(user_id, key);
        if let Some(cached) = self.cache.get::<UserSetting>(&ck).await? {
            return Ok(cached);
        }

        let setting = self
            .repo
            .find(user_id, key)
            .await?
            .ok_or(UserSettingDomainError::NotFound)?;

        self.cache.set(&ck, &setting, CACHE_TTL).await?;
        Ok(setting)
    }

    async fn upsert(
        &self,
        user_id: i32,
        key: &str,
        req: UpsertUserSettingRequest,
    ) -> Result<UserSetting, AppError> {
        if key.trim().is_empty() {
            return Err(UserSettingDomainError::InvalidKey.into());
        }

        let old = self.repo.find(user_id, key).await.ok().flatten();

        let setting = self.repo.upsert(user_id, key, req.value.as_deref()).await?;

        spawn_audit_log(
            self.audit.clone(),
            user_id,
            "user_setting.upsert",
            key,
            old.as_ref(),
            Some(&setting),
        );

        self.invalidate(user_id, key).await?;
        Ok(setting)
    }

    async fn delete(&self, user_id: i32, key: &str) -> Result<(), AppError> {
        let old = self.repo.find(user_id, key).await.ok().flatten();

        self.repo.delete(user_id, key).await?;

        spawn_audit_log(
            self.audit.clone(),
            user_id,
            "user_setting.delete",
            key,
            old.as_ref(),
            None,
        );

        self.invalidate(user_id, key).await?;
        Ok(())
    }
}
