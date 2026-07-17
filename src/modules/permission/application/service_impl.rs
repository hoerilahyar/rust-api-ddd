use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;

use chrono::Utc;

use crate::modules::log_audit_trails::domain::entity::AuditTrailLog;
use crate::modules::permission::application::service::PermissionService;
use crate::modules::permission::application::{CreatePermissionRequest, UpdatePermissionRequest};
use crate::modules::permission::domain::{Name, Permission, PermissionRepository};
use crate::shared::cache::{CacheRepository, RedisCacheRepository};
use crate::shared::context::current_request_context;
use crate::shared::contracts::AuditTrailRecorder;
use crate::shared::domain::PaginationParams;
use crate::shared::errors::AppError;

const ENTITY_TYPE: &str = "permission";

/// Fires an audit trail write in the background so a slow/unavailable audit
/// sink never blocks or fails the actual permission mutation.
fn spawn_audit_log(
    audit: Arc<dyn AuditTrailRecorder>,
    actor_id: i32,
    action: &'static str,
    permission_id: i32,
    old_values: Option<&Permission>,
    new_values: Option<&Permission>,
) {
    let ctx = current_request_context();
    let log = AuditTrailLog {
        id: 0,
        user_id: Some(actor_id),
        action: action.to_string(),
        entity_type: ENTITY_TYPE.to_string(),
        entity_id: Some(permission_id.to_string()),
        old_values: old_values.and_then(|p| serde_json::to_value(p).ok()),
        new_values: new_values.and_then(|p| serde_json::to_value(p).ok()),
        ip_address: ctx.ip_address,
        user_agent: ctx.user_agent,
        description: Some(format!("permission id {permission_id}")),
        created_at: Utc::now(),
    };

    tokio::spawn(async move {
        if let Err(err) = audit.record_audit_trail_log(log).await {
            tracing::error!(error = ?err, permission_id, action, "failed to record permission audit trail log");
        }
    });
}

const CACHE_TTL: Duration = Duration::from_secs(300);

fn cache_key(id: i32) -> String {
    format!("permission:id:{id}")
}

pub struct PermissionServiceImpl {
    audit: Arc<dyn AuditTrailRecorder>,
    repo: Arc<dyn PermissionRepository>,
    cache: Arc<RedisCacheRepository>,
}

impl PermissionServiceImpl {
    pub fn new(
        audit: Arc<dyn AuditTrailRecorder>,
        repo: Arc<dyn PermissionRepository>,
        cache: Arc<RedisCacheRepository>,
    ) -> Self {
        Self { audit, repo, cache }
    }
}

#[async_trait]
impl PermissionService for PermissionServiceImpl {
    async fn get_by_id(&self, id: i32) -> Result<Permission, AppError> {
        let key = cache_key(id);
        if let Some(cached) = self.cache.get::<Permission>(&key).await? {
            return Ok(cached);
        }

        let permission = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("permission not found".to_string()))?;

        self.cache.set(&key, &permission, CACHE_TTL).await?;
        Ok(permission)
    }

    async fn list(
        &self,
        pagination: &PaginationParams,
    ) -> Result<(Vec<Permission>, i64), AppError> {
        self.repo.list(pagination).await
    }

    async fn create(
        &self,
        req: CreatePermissionRequest,
        actor_id: i32,
    ) -> Result<Permission, AppError> {
        Name::parse(&req.name)?;

        if self.repo.find_by_name(&req.name).await?.is_some() {
            return Err(AppError::Conflict("name is already registered".to_string()));
        }

        let permission: Permission = self
            .repo
            .create(&req.name, req.description.as_deref())
            .await?;

        spawn_audit_log(
            self.audit.clone(),
            actor_id,
            "permission.create",
            permission.id,
            None,
            Some(&permission),
        );

        Ok(permission)
    }
    async fn update(
        &self,
        id: i32,
        req: UpdatePermissionRequest,
        actor_id: i32,
    ) -> Result<Permission, AppError> {
        if let Some(name) = &req.name {
            Name::parse(name)?;
            if let Some(existing) = self.repo.find_by_name(name).await? {
                if existing.id != id {
                    return Err(AppError::Conflict("name is already registered".to_string()));
                }
            }
        }

        let existing = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("permission not found".to_string()))?;

        let permission = self
            .repo
            .update(id, req.name.as_deref(), req.description.as_deref())
            .await?;

        spawn_audit_log(
            self.audit.clone(),
            actor_id,
            "permission.update",
            id,
            Some(&existing),
            Some(&permission),
        );

        self.cache.delete(&cache_key(id)).await?;
        Ok(permission)
    }

    async fn delete(&self, id: i32, actor_id: i32) -> Result<(), AppError> {
        let existing = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("permission not found".to_string()))?;

        self.repo.delete(id).await?;

        spawn_audit_log(
            self.audit.clone(),
            actor_id,
            "permission.delete",
            id,
            Some(&existing),
            None,
        );

        self.cache.delete(&cache_key(id)).await?;
        Ok(())
    }
}
