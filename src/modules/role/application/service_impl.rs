use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;

use chrono::Utc;

use crate::modules::log_audit_trails::domain::entity::AuditTrailLog;
use crate::modules::role::application::service::RoleService;
use crate::modules::role::application::{CreateRoleRequest, UpdateRoleRequest};
use crate::modules::role::domain::{Name, Role, RoleRepository};
use crate::shared::cache::{CacheRepository, RedisCacheRepository};
use crate::shared::context::current_request_context;
use crate::shared::contracts::AuditTrailRecorder;
use crate::shared::domain::PaginationParams;
use crate::shared::errors::AppError;

const ENTITY_TYPE: &str = "role";

/// Fires an audit trail write in the background so a slow/unavailable audit
/// sink never blocks or fails the actual role mutation.
fn spawn_audit_log(
    audit: Arc<dyn AuditTrailRecorder>,
    actor_id: i32,
    action: &'static str,
    role_id: i32,
    old_values: Option<&Role>,
    new_values: Option<&Role>,
) {
    let ctx = current_request_context();
    let log = AuditTrailLog {
        id: 0,
        user_id: Some(actor_id),
        action: action.to_string(),
        entity_type: ENTITY_TYPE.to_string(),
        entity_id: Some(role_id.to_string()),
        old_values: old_values.and_then(|r| serde_json::to_value(r).ok()),
        new_values: new_values.and_then(|r| serde_json::to_value(r).ok()),
        ip_address: ctx.ip_address,
        user_agent: ctx.user_agent,
        description: Some(format!("role id {role_id}")),
        created_at: Utc::now(),
    };

    tokio::spawn(async move {
        if let Err(err) = audit.record_audit_trail_log(log).await {
            tracing::error!(error = ?err, role_id, action, "failed to record role audit trail log");
        }
    });
}

const CACHE_TTL: Duration = Duration::from_secs(300);

fn cache_key(id: i32) -> String {
    format!("role:id:{id}")
}

pub struct RoleServiceImpl {
    audit: Arc<dyn AuditTrailRecorder>,
    repo: Arc<dyn RoleRepository>,
    cache: Arc<RedisCacheRepository>,
}

impl RoleServiceImpl {
    pub fn new(
        audit: Arc<dyn AuditTrailRecorder>,
        repo: Arc<dyn RoleRepository>,
        cache: Arc<RedisCacheRepository>,
    ) -> Self {
        Self { audit, repo, cache }
    }
}

#[async_trait]
impl RoleService for RoleServiceImpl {
    async fn get_by_id(&self, id: i32) -> Result<Role, AppError> {
        let key = cache_key(id);
        if let Some(cached) = self.cache.get::<Role>(&key).await? {
            return Ok(cached);
        }

        let role = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("role not found".to_string()))?;

        self.cache.set(&key, &role, CACHE_TTL).await?;
        Ok(role)
    }

    async fn list(&self, pagination: &PaginationParams) -> Result<(Vec<Role>, i64), AppError> {
        self.repo.list(pagination).await
    }

    async fn create(&self, req: CreateRoleRequest, actor_id: i32) -> Result<Role, AppError> {
        Name::parse(&req.name)?;

        if self.repo.find_by_name(&req.name).await?.is_some() {
            return Err(AppError::Conflict("name is already registered".to_string()));
        }

        let role = self
            .repo
            .create(&req.name, req.description.as_deref())
            .await?;

        spawn_audit_log(
            self.audit.clone(),
            actor_id,
            "role.create",
            role.id,
            None,
            Some(&role),
        );

        Ok(role)
    }
    async fn update(
        &self,
        id: i32,
        req: UpdateRoleRequest,
        actor_id: i32,
    ) -> Result<Role, AppError> {
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
            .ok_or_else(|| AppError::NotFound("role not found".to_string()))?;

        let role = self
            .repo
            .update(id, req.name.as_deref(), req.description.as_deref())
            .await?;

        spawn_audit_log(
            self.audit.clone(),
            actor_id,
            "role.update",
            id,
            Some(&existing),
            Some(&role),
        );

        self.cache.delete(&cache_key(id)).await?;
        Ok(role)
    }

    async fn delete(&self, id: i32, actor_id: i32) -> Result<(), AppError> {
        let existing = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("role not found".to_string()))?;

        self.repo.delete(id).await?;

        spawn_audit_log(
            self.audit.clone(),
            actor_id,
            "role.delete",
            id,
            Some(&existing),
            None,
        );

        self.cache.delete(&cache_key(id)).await?;
        Ok(())
    }

    async fn assign_permission(&self, role_id: i32, permission_name: &str) -> Result<(), AppError> {
        let (permission_id, _) = self
            .repo
            .find_permission_by_name(permission_name)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!("permission '{permission_name}' not found"))
            })?;

        self.repo.assign_permission(role_id, permission_id).await?;
        self.cache.delete(&cache_key(role_id)).await?;
        Ok(())
    }
    async fn revoke_permission(&self, role_id: i32, permission_name: &str) -> Result<(), AppError> {
        let (permission_id, _) = self
            .repo
            .find_permission_by_name(permission_name)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!("permission '{permission_name}' not found"))
            })?;

        self.repo.revoke_permission(role_id, permission_id).await?;
        self.cache.delete(&cache_key(role_id)).await?;
        Ok(())
    }
}
