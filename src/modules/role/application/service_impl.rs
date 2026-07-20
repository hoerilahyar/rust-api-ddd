use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;

use chrono::Utc;

use crate::modules::log_audit_trails::domain::entity::AuditTrailLog;
use crate::modules::permission::domain::PermissionRepository;
use crate::modules::role::application::service::RoleService;
use crate::modules::role::application::{CreateRoleRequest, UpdateRoleRequest};
use crate::modules::role::domain::{Name, Role, RoleRepository};
use crate::modules::user::domain::repository::UserRepository;
use crate::shared::authz_cache;
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
    /// Needed to look up every user holding a given role, so their live
    /// authz snapshot can be invalidated whenever that role's permissions
    /// change -- see `invalidate_authz_for_role`.
    user_repo: Arc<dyn UserRepository>,
    /// Used to validate a `permission_id` exists before assigning/revoking
    /// it, so an unknown id surfaces as a clean 404 instead of a raw FK
    /// violation from the `role_permissions` insert.
    permission_repo: Arc<dyn PermissionRepository>,
}

impl RoleServiceImpl {
    pub fn new(
        audit: Arc<dyn AuditTrailRecorder>,
        repo: Arc<dyn RoleRepository>,
        cache: Arc<RedisCacheRepository>,
        user_repo: Arc<dyn UserRepository>,
        permission_repo: Arc<dyn PermissionRepository>,
    ) -> Self {
        Self {
            audit,
            repo,
            cache,
            user_repo,
            permission_repo,
        }
    }

    /// Invalidates the live authz snapshot (see `shared::authz_cache`) of
    /// every user currently holding `role_id`, so a permission grant or
    /// revoke on this role is visible on each of their very next requests
    /// instead of waiting out the snapshot's safety-net TTL. Their access
    /// tokens are left alone -- no forced logout, in either direction.
    /// Best-effort per user: a failure for one user is logged and does not
    /// stop the others from being invalidated.
    async fn invalidate_authz_for_role(&self, role_id: i32) -> Result<(), AppError> {
        let user_ids = self.user_repo.find_user_ids_by_role(role_id).await?;
        for user_id in user_ids {
            if let Err(err) = authz_cache::invalidate(self.cache.as_ref(), user_id).await {
                tracing::error!(
                    error = ?err,
                    role_id,
                    user_id,
                    "failed to invalidate live authz snapshot after role permission change"
                );
            }
        }
        Ok(())
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

        // Capture affected users before the role (and its user_roles rows,
        // if the schema cascades) disappears.
        let affected_user_ids = self.user_repo.find_user_ids_by_role(id).await?;

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

        for user_id in affected_user_ids {
            if let Err(err) = authz_cache::invalidate(self.cache.as_ref(), user_id).await {
                tracing::error!(
                    error = ?err,
                    role_id = id,
                    user_id,
                    "failed to invalidate live authz snapshot after role deletion"
                );
            }
        }

        Ok(())
    }

    /// Validates every id in `permission_ids` exists, reconciles the role's
    /// permission set to exactly that list in one DB transaction, then
    /// invalidates the live authz snapshot for every user holding this role
    /// so the change is visible on their very next request.
    async fn sync_permissions(&self, role_id: i32, permission_ids: &[i32]) -> Result<(), AppError> {
        let mut unique_ids: Vec<i32> = permission_ids.to_vec();
        unique_ids.sort_unstable();
        unique_ids.dedup();

        let found = self.permission_repo.find_many_by_ids(&unique_ids).await?;
        if found.len() != unique_ids.len() {
            let found_ids: std::collections::HashSet<i32> = found.iter().map(|p| p.id).collect();
            let missing: Vec<String> = unique_ids
                .iter()
                .filter(|id| !found_ids.contains(id))
                .map(|id| id.to_string())
                .collect();
            return Err(AppError::NotFound(format!(
                "permission(s) not found: {}",
                missing.join(", ")
            )));
        }

        self.repo.sync_permissions(role_id, &unique_ids).await?;
        self.cache.delete(&cache_key(role_id)).await?;

        // Every user holding this role may have gained or lost a
        // permission/menu. Invalidate their live authz snapshot so
        // `require_auth` picks up the new permission set on their very
        // next request -- their current access token keeps working, it
        // just reflects the updated permission set.
        self.invalidate_authz_for_role(role_id).await?;

        Ok(())
    }
}
