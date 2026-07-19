use std::sync::Arc;
use std::time::Duration;

use argon2::password_hash::{
    rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
};
use argon2::Argon2;
use async_trait::async_trait;
use chrono::Utc;

use crate::modules::log_audit_trails::domain::AuditTrailLog;
use crate::modules::user::application::dto::{CreateUserRequest, UpdateUserRequest};
use crate::modules::user::application::service::UserService;
use crate::modules::user::domain::entity::User;
use crate::modules::user::domain::repository::{PasswordHistoryRepository, UserRepository};
use crate::modules::user::domain::value_object::{Email, Username};
use crate::shared::cache::{CacheRepository, RedisCacheRepository};
use crate::shared::context::current_request_context;
use crate::shared::contracts::AuditTrailRecorder;
use crate::shared::domain::PaginationParams;
use crate::shared::errors::AppError;

const ENTITY_TYPE: &str = "users";

/// Fires an audit trail write in the background so a slow/unavailable audit
/// sink never blocks or fails the actual mutation. `user_id` is both
/// the subject and the actor here -- every method in this module is
/// self-service, scoped to the caller's own `Claims::sub`.
fn spawn_audit_log(
    audit: Arc<dyn AuditTrailRecorder>,
    user_id: i32,
    action: &'static str,
    key: &str,
    old_values: Option<&User>,
    new_values: Option<&User>,
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
        description: Some(format!("user {user_id} update data user {key}")),
        created_at: Utc::now(),
    };

    let key = key.to_string();
    tokio::spawn(async move {
        if let Err(err) = audit.record_audit_trail_log(log).await {
            tracing::error!(error = ?err, user_id, key, action, "failed to record user audit trail log");
        }
    });
}

const CACHE_TTL: Duration = Duration::from_secs(300);
const PASSWORD_HISTORY_LIMIT: i64 = 3;

fn cache_key(id: i32) -> String {
    format!("user:id:{id}")
}

pub struct UserServiceImpl {
    audit: Arc<dyn AuditTrailRecorder>,
    repo: Arc<dyn UserRepository>,
    password_history_repo: Arc<dyn PasswordHistoryRepository>,
    cache: Arc<RedisCacheRepository>,
}

impl UserServiceImpl {
    pub fn new(
        audit: Arc<dyn AuditTrailRecorder>,
        repo: Arc<dyn UserRepository>,
        password_history_repo: Arc<dyn PasswordHistoryRepository>,
        cache: Arc<RedisCacheRepository>,
    ) -> Self {
        Self {
            audit,
            repo,
            password_history_repo,
            cache,
        }
    }

    fn hash_password(&self, plain: &str) -> Result<String, AppError> {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(plain.as_bytes(), &salt)
            .map(|h| h.to_string())
            .map_err(|e| AppError::Internal(anyhow::anyhow!("failed to hash password: {e}")))
    }

    fn verify_password(&self, plain: &str, hash: &str) -> Result<bool, AppError> {
        let parsed = PasswordHash::new(hash)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("invalid password hash: {e}")))?;
        Ok(Argon2::default()
            .verify_password(plain.as_bytes(), &parsed)
            .is_ok())
    }

    async fn ensure_password_not_reused(
        &self,
        id: i32,
        new_password: &str,
        current_hash: &str,
    ) -> Result<(), AppError> {
        if self.verify_password(new_password, current_hash)? {
            return Err(AppError::BadRequest(
                "new password must be different from current password".into(),
            ));
        }

        let recent_hashes = self
            .password_history_repo
            .recent_password_hashes(id, PASSWORD_HISTORY_LIMIT)
            .await?;

        for hash in &recent_hashes {
            if self.verify_password(new_password, hash)? {
                return Err(AppError::BadRequest(format!(
                    "new password must not match any of your last {PASSWORD_HISTORY_LIMIT} passwords"
                )));
            }
        }

        Ok(())
    }

    async fn record_password_history(&self, id: i32, old_hash: &str) -> Result<(), AppError> {
        self.password_history_repo
            .record_password_hash(id, old_hash)
            .await?;
        self.password_history_repo
            .prune_password_history(id, PASSWORD_HISTORY_LIMIT)
            .await?;
        Ok(())
    }
}

#[async_trait]
impl UserService for UserServiceImpl {
    async fn get_by_id(&self, id: i32) -> Result<User, AppError> {
        let key = cache_key(id);
        if let Some(cached) = self.cache.get::<User>(&key).await? {
            return Ok(cached);
        }

        let user = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("user not found".to_string()))?;

        self.cache.set(&key, &user, CACHE_TTL).await?;
        Ok(user)
    }

    async fn list(&self, pagination: &PaginationParams) -> Result<(Vec<User>, i64), AppError> {
        self.repo.list(pagination).await
    }

    async fn create(&self, req: CreateUserRequest, actor_id: i32) -> Result<User, AppError> {
        Email::parse(&req.email)?;
        Username::parse(&req.username)?;

        if self.repo.find_by_email(&req.email).await?.is_some() {
            return Err(AppError::Conflict(
                "email is already registered".to_string(),
            ));
        }
        if self.repo.find_by_username(&req.username).await?.is_some() {
            return Err(AppError::Conflict("username is already taken".to_string()));
        }

        let password_hash = self.hash_password(&req.password)?;
        let user = self
            .repo
            .create(&req.name, &req.username, &req.email, &password_hash)
            .await?;

        spawn_audit_log(
            self.audit.clone(),
            actor_id,
            "user.create",
            &user.id.to_string(),
            None,
            Some(&user),
        );

        Ok(user)
    }

    async fn update(
        &self,
        id: i32,
        req: UpdateUserRequest,
        actor_id: i32,
    ) -> Result<User, AppError> {
        if let Some(email) = &req.email {
            Email::parse(email)?;
            if let Some(existing) = self.repo.find_by_email(email).await? {
                if existing.id != id {
                    return Err(AppError::Conflict(
                        "email is already registered".to_string(),
                    ));
                }
            }
        }

        if let Some(username) = &req.username {
            Username::parse(username)?;
            if let Some(existing) = self.repo.find_by_username(username).await? {
                if existing.id != id {
                    return Err(AppError::Conflict(
                        "username is already registered".to_string(),
                    ));
                }
            }
        }

        let existing = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("user not found".to_string()))?;

        let new_password_hash = if let Some(password) = &req.password {
            self.ensure_password_not_reused(id, password, &existing.password_hash)
                .await?;

            Some(self.hash_password(password)?)
        } else {
            None
        };

        let user = self
            .repo
            .update(
                id,
                req.name.as_deref(),
                req.username.as_deref(),
                req.email.as_deref(),
                new_password_hash.as_deref(),
                req.is_active,
            )
            .await?;

        if new_password_hash.is_some() {
            self.record_password_history(id, &existing.password_hash)
                .await?;
        }

        spawn_audit_log(
            self.audit.clone(),
            actor_id,
            "user.update",
            &id.to_string(),
            Some(&existing),
            Some(&user),
        );

        self.cache.delete(&cache_key(id)).await?;
        Ok(user)
    }

    async fn change_password(
        &self,
        id: i32,
        current_password: &str,
        new_password: &str,
        actor_id: i32,
    ) -> Result<(), AppError> {
        let user = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("user not found".to_string()))?;

        if !self.verify_password(current_password, &user.password_hash)? {
            return Err(AppError::Unauthorized(
                "current password is incorrect".to_string(),
            ));
        }

        self.ensure_password_not_reused(id, new_password, &user.password_hash)
            .await?;

        let new_hash = self.hash_password(new_password)?;
        self.repo.update_password(id, &new_hash).await?;
        self.record_password_history(id, &user.password_hash)
            .await?;

        spawn_audit_log(
            self.audit.clone(),
            actor_id,
            "user.change_password",
            &id.to_string(),
            Some(&user),
            None,
        );

        self.cache.delete(&cache_key(id)).await?;
        Ok(())
    }

    async fn delete(&self, id: i32, actor_id: i32) -> Result<(), AppError> {
        let existing = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("user not found".to_string()))?;

        self.repo.soft_delete(id).await?;

        spawn_audit_log(
            self.audit.clone(),
            actor_id,
            "user.delete",
            &id.to_string(),
            Some(&existing),
            None,
        );

        self.cache.delete(&cache_key(id)).await?;
        Ok(())
    }

    async fn assign_role(
        &self,
        user_id: i32,
        role_name: &str,
        assigned_by: Option<i32>,
        actor_id: i32,
    ) -> Result<(), AppError> {
        let (role_id, _) = self
            .repo
            .find_role_by_name(role_name)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("role '{role_name}' not found")))?;

        let existing = self
            .repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("user not found".to_string()))?;

        self.repo.assign_role(user_id, role_id, assigned_by).await?;

        let updated = self
            .repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("user not found".to_string()))?;

        spawn_audit_log(
            self.audit.clone(),
            actor_id,
            "user.assign_role",
            &user_id.to_string(),
            Some(&existing),
            Some(&updated),
        );

        self.cache.delete(&cache_key(user_id)).await?;
        Ok(())
    }

    async fn revoke_role(
        &self,
        user_id: i32,
        role_name: &str,
        actor_id: i32,
    ) -> Result<(), AppError> {
        let (role_id, _) = self
            .repo
            .find_role_by_name(role_name)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("role '{role_name}' not found")))?;

        let existing = self
            .repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("user not found".to_string()))?;

        self.repo.revoke_role(user_id, role_id).await?;

        let updated = self
            .repo
            .find_by_id(user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("user not found".to_string()))?;

        spawn_audit_log(
            self.audit.clone(),
            actor_id,
            "user.revoke_role",
            &user_id.to_string(),
            Some(&existing),
            Some(&updated),
        );

        self.cache.delete(&cache_key(user_id)).await?;
        Ok(())
    }
}
