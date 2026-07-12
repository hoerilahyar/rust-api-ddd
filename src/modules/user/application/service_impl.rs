use std::sync::Arc;
use std::time::Duration;

use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng};
use argon2::Argon2;
use async_trait::async_trait;

use crate::modules::user::application::dto::{CreateUserRequest, UpdateUserRequest};
use crate::modules::user::application::service::UserService;
use crate::modules::user::domain::entity::User;
use crate::modules::user::domain::repository::UserRepository;
use crate::modules::user::domain::value_object::{Email, Username};
use crate::shared::cache::{CacheRepository, RedisCacheRepository};
use crate::shared::domain::PaginationParams;
use crate::shared::errors::AppError;

const CACHE_TTL: Duration = Duration::from_secs(300);

fn cache_key(id: i32) -> String {
    format!("user:id:{id}")
}

pub struct UserServiceImpl {
    repo: Arc<dyn UserRepository>,
    cache: Arc<RedisCacheRepository>,
}

impl UserServiceImpl {
    pub fn new(repo: Arc<dyn UserRepository>, cache: Arc<RedisCacheRepository>) -> Self {
        Self { repo, cache }
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
        Ok(Argon2::default().verify_password(plain.as_bytes(), &parsed).is_ok())
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

    async fn create(&self, req: CreateUserRequest) -> Result<User, AppError> {
        Email::parse(&req.email)?;
        Username::parse(&req.username)?;

        if self.repo.find_by_email(&req.email).await?.is_some() {
            return Err(AppError::Conflict("email is already registered".to_string()));
        }
        if self.repo.find_by_username(&req.username).await?.is_some() {
            return Err(AppError::Conflict("username is already taken".to_string()));
        }

        let password_hash = self.hash_password(&req.password)?;
        let user = self
            .repo
            .create(&req.name, &req.username, &req.email, &password_hash)
            .await?;

        Ok(user)
    }

    async fn update(&self, id: i32, req: UpdateUserRequest) -> Result<User, AppError> {
        if let Some(email) = &req.email {
            Email::parse(email)?;
            if let Some(existing) = self.repo.find_by_email(email).await? {
                if existing.id != id {
                    return Err(AppError::Conflict("email is already registered".to_string()));
                }
            }
        }

        let user = self
            .repo
            .update(id, req.name.as_deref(), req.email.as_deref(), req.is_active)
            .await?;

        self.cache.delete(&cache_key(id)).await?;
        Ok(user)
    }

    async fn change_password(&self, id: i32, current_password: &str, new_password: &str) -> Result<(), AppError> {
        let user = self
            .repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("user not found".to_string()))?;

        if !self.verify_password(current_password, &user.password_hash)? {
            return Err(AppError::Unauthorized("current password is incorrect".to_string()));
        }

        let new_hash = self.hash_password(new_password)?;
        self.repo.update_password(id, &new_hash).await?;
        self.cache.delete(&cache_key(id)).await?;
        Ok(())
    }

    async fn delete(&self, id: i32) -> Result<(), AppError> {
        self.repo.soft_delete(id).await?;
        self.cache.delete(&cache_key(id)).await?;
        Ok(())
    }

    async fn assign_role(&self, user_id: i32, role_name: &str, assigned_by: Option<i32>) -> Result<(), AppError> {
        let (role_id, _) = self
            .repo
            .find_role_by_name(role_name)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("role '{role_name}' not found")))?;

        self.repo.assign_role(user_id, role_id, assigned_by).await?;
        self.cache.delete(&cache_key(user_id)).await?;
        Ok(())
    }

    async fn revoke_role(&self, user_id: i32, role_name: &str) -> Result<(), AppError> {
        let (role_id, _) = self
            .repo
            .find_role_by_name(role_name)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("role '{role_name}' not found")))?;

        self.repo.revoke_role(user_id, role_id).await?;
        self.cache.delete(&cache_key(user_id)).await?;
        Ok(())
    }
}
