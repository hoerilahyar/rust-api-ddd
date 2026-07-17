use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;

use chrono::Utc;

use crate::modules::log_audit_trails::domain::entity::AuditTrailLog;
use crate::modules::setting::application::dto::UpsertSettingRequest;
use crate::modules::setting::application::service::SettingService;
use crate::modules::setting::domain::{SettingDomainError, SettingRepository, SystemSetting};
use crate::shared::cache::{CacheRepository, RedisCacheRepository};
use crate::shared::context::current_request_context;
use crate::shared::contracts::AuditTrailRecorder;
use crate::shared::errors::AppError;

const ENTITY_TYPE: &str = "system_setting";

/// Fires an audit trail write in the background so a slow/unavailable audit
/// sink never blocks or fails the actual setting mutation.
fn spawn_audit_log(
    audit: Arc<dyn AuditTrailRecorder>,
    actor_id: i32,
    action: &'static str,
    key: &str,
    old_values: Option<&SystemSetting>,
    new_values: Option<&SystemSetting>,
) {
    let ctx = current_request_context();
    let log = AuditTrailLog {
        id: 0,
        user_id: Some(actor_id),
        action: action.to_string(),
        entity_type: ENTITY_TYPE.to_string(),
        entity_id: Some(key.to_string()),
        old_values: old_values.and_then(|s| serde_json::to_value(s).ok()),
        new_values: new_values.and_then(|s| serde_json::to_value(s).ok()),
        ip_address: ctx.ip_address,
        user_agent: ctx.user_agent,
        description: Some(format!("setting key {key}")),
        created_at: Utc::now(),
    };

    let key = key.to_string();
    tokio::spawn(async move {
        if let Err(err) = audit.record_audit_trail_log(log).await {
            tracing::error!(error = ?err, key, action, "failed to record setting audit trail log");
        }
    });
}

const CACHE_TTL: Duration = Duration::from_secs(300);
const LIST_CACHE_KEY: &str = "setting:list";

fn cache_key(key: &str) -> String {
    format!("setting:key:{key}")
}

pub struct SettingServiceImpl {
    audit: Arc<dyn AuditTrailRecorder>,
    repo: Arc<dyn SettingRepository>,
    cache: Arc<RedisCacheRepository>,
}

impl SettingServiceImpl {
    pub fn new(
        audit: Arc<dyn AuditTrailRecorder>,
        repo: Arc<dyn SettingRepository>,
        cache: Arc<RedisCacheRepository>,
    ) -> Self {
        Self { audit, repo, cache }
    }

    async fn invalidate(&self, key: &str) -> Result<(), AppError> {
        self.cache.delete(&cache_key(key)).await?;
        self.cache.delete(LIST_CACHE_KEY).await?;
        Ok(())
    }
}

#[async_trait]
impl SettingService for SettingServiceImpl {
    async fn list(&self) -> Result<Vec<SystemSetting>, AppError> {
        if let Some(cached) = self.cache.get::<Vec<SystemSetting>>(LIST_CACHE_KEY).await? {
            return Ok(cached);
        }

        let settings = self.repo.list_all().await?;
        self.cache.set(LIST_CACHE_KEY, &settings, CACHE_TTL).await?;
        Ok(settings)
    }

    async fn get_by_key(&self, key: &str) -> Result<SystemSetting, AppError> {
        let ck = cache_key(key);
        if let Some(cached) = self.cache.get::<SystemSetting>(&ck).await? {
            return Ok(cached);
        }

        let setting = self
            .repo
            .find_by_key(key)
            .await?
            .ok_or(SettingDomainError::NotFound)?;

        self.cache.set(&ck, &setting, CACHE_TTL).await?;
        Ok(setting)
    }

    async fn upsert(
        &self,
        key: &str,
        req: UpsertSettingRequest,
        updated_by: i32,
    ) -> Result<SystemSetting, AppError> {
        if key.trim().is_empty() {
            return Err(SettingDomainError::InvalidKey.into());
        }

        let old = self.repo.find_by_key(key).await.ok().flatten();

        let setting = self
            .repo
            .upsert(
                key,
                req.value.as_deref(),
                req.description.as_deref(),
                Some(updated_by),
            )
            .await?;

        spawn_audit_log(
            self.audit.clone(),
            updated_by,
            "setting.upsert",
            key,
            old.as_ref(),
            Some(&setting),
        );

        self.invalidate(key).await?;
        Ok(setting)
    }

    async fn delete(&self, key: &str, actor_id: i32) -> Result<(), AppError> {
        let old = self.repo.find_by_key(key).await.ok().flatten();

        self.repo.delete(key).await?;

        spawn_audit_log(
            self.audit.clone(),
            actor_id,
            "setting.delete",
            key,
            old.as_ref(),
            None,
        );

        self.invalidate(key).await?;
        Ok(())
    }
}
