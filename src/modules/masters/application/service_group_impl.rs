use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;

use chrono::Utc;

use crate::modules::audit_trail_log::domain::entity::AuditTrailLog;
use crate::modules::masters::application::{
    CreateMasterGroupRequest, MasterGroupService, UpdateMasterGroupRequest,
};
use crate::modules::masters::domain::{MasterGroup, MasterGroupRepository, Name};
use crate::shared::cache::{CacheRepository, RedisCacheRepository};
use crate::shared::context::current_request_context;
use crate::shared::contracts::AuditTrailRecorder;
use crate::shared::domain::PaginationParams;
use crate::shared::errors::AppError;

const ENTITY_TYPE: &str = "master_group";

/// Fires an audit trail write in the background so a slow/unavailable audit
/// sink never blocks or fails the actual master group mutation.
fn spawn_audit_log(
    audit: Arc<dyn AuditTrailRecorder>,
    actor_id: i32,
    action: &'static str,
    master_id: i64,
    old_values: Option<&MasterGroup>,
    new_values: Option<&MasterGroup>,
) {
    let ctx = current_request_context();
    let log = AuditTrailLog {
        id: 0,
        user_id: Some(actor_id),
        action: action.to_string(),
        entity_type: ENTITY_TYPE.to_string(),
        entity_id: Some(master_id.to_string()),
        old_values: old_values.and_then(|r| serde_json::to_value(r).ok()),
        new_values: new_values.and_then(|r| serde_json::to_value(r).ok()),
        ip_address: ctx.ip_address,
        user_agent: ctx.user_agent,
        description: Some(format!("master group id {master_id}")),
        created_at: Utc::now(),
    };

    tokio::spawn(async move {
        if let Err(err) = audit.record_audit_trail_log(log).await {
            tracing::error!(error = ?err, master_id, action, "failed to record master group audit trail log");
        }
    });
}

const CACHE_TTL: Duration = Duration::from_secs(300);

fn cache_key(id: i64) -> String {
    format!("master_group:id:{id}")
}

pub struct MasterGroupServiceImpl {
    audit: Arc<dyn AuditTrailRecorder>,
    repo: Arc<dyn MasterGroupRepository>,
    cache: Arc<RedisCacheRepository>,
}

impl MasterGroupServiceImpl {
    pub fn new(
        audit: Arc<dyn AuditTrailRecorder>,
        repo: Arc<dyn MasterGroupRepository>,
        cache: Arc<RedisCacheRepository>,
    ) -> Self {
        Self { audit, repo, cache }
    }
}

#[async_trait]
impl MasterGroupService for MasterGroupServiceImpl {
    async fn get_by_id(&self, id: i64) -> Result<MasterGroup, AppError> {
        let key = cache_key(id);
        if let Some(cached) = self.cache.get::<MasterGroup>(&key).await? {
            return Ok(cached);
        }

        let group = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("master group not found".to_string()))?;

        self.cache.set(&key, &group, CACHE_TTL).await?;
        Ok(group)
    }

    async fn list(
        &self,
        pagination: &PaginationParams,
    ) -> Result<(Vec<MasterGroup>, i64), AppError> {
        self.repo.list(pagination).await
    }

    async fn create(
        &self,
        req: CreateMasterGroupRequest,
        actor_id: i32,
    ) -> Result<MasterGroup, AppError> {
        Name::parse(&req.name)?;

        if self.repo.find_by_code(&req.code).await?.is_some() {
            return Err(AppError::Conflict("code is already registered".to_string()));
        }
        if self.repo.find_by_name(&req.name).await?.is_some() {
            return Err(AppError::Conflict("name is already registered".to_string()));
        }

        let group = self
            .repo
            .create(&req.code, &req.name, req.description.as_deref())
            .await?;

        spawn_audit_log(
            self.audit.clone(),
            actor_id,
            "master_group.create",
            group.id,
            None,
            Some(&group),
        );
        Ok(group)
    }

    async fn update(
        &self,
        id: i64,
        req: UpdateMasterGroupRequest,
        actor_id: i32,
    ) -> Result<MasterGroup, AppError> {
        Name::parse(&req.name)?;

        if let Some(existing) = self.repo.find_by_code(&req.code).await? {
            if existing.id != id {
                return Err(AppError::Conflict("code is already registered".to_string()));
            }
        }
        if let Some(existing) = self.repo.find_by_name(&req.name).await? {
            if existing.id != id {
                return Err(AppError::Conflict("name is already registered".to_string()));
            }
        }

        let existing = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("master group not found".to_string()))?;

        let group = self
            .repo
            .update(
                id,
                Some(req.code.as_str()),
                Some(req.name.as_str()),
                req.description.as_deref(),
                Some(&req.is_active),
            )
            .await?;

        spawn_audit_log(
            self.audit.clone(),
            actor_id,
            "master_group.update",
            id,
            Some(&existing),
            Some(&group),
        );
        self.cache.delete(&cache_key(id)).await?;
        Ok(group)
    }

    async fn delete(&self, id: i64, actor_id: i32) -> Result<(), AppError> {
        let existing = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("master group not found".to_string()))?;

        self.repo.delete(id).await?;

        spawn_audit_log(
            self.audit.clone(),
            actor_id,
            "master_group.delete",
            id,
            Some(&existing),
            None,
        );

        self.cache.delete(&cache_key(id)).await?;
        Ok(())
    }
}
