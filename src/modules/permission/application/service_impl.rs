use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;

use crate::modules::permission::application::service::PermissionService;
use crate::modules::permission::application::{CreatePermissionRequest, UpdatePermissionRequest};
use crate::modules::permission::domain::{Name, Permission, PermissionRepository};
use crate::shared::cache::{CacheRepository, RedisCacheRepository};
use crate::shared::domain::PaginationParams;
use crate::shared::errors::AppError;

const CACHE_TTL: Duration = Duration::from_secs(300);

fn cache_key(id: i32) -> String {
    format!("permission:id:{id}")
}

pub struct PermissionServiceImpl {
    repo: Arc<dyn PermissionRepository>,
    cache: Arc<RedisCacheRepository>,
}

impl PermissionServiceImpl {
    pub fn new(repo: Arc<dyn PermissionRepository>, cache: Arc<RedisCacheRepository>) -> Self {
        Self { repo, cache }
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

    async fn create(&self, req: CreatePermissionRequest) -> Result<Permission, AppError> {
        Name::parse(&req.name)?;

        if self.repo.find_by_name(&req.name).await?.is_some() {
            return Err(AppError::Conflict("name is already registered".to_string()));
        }

        let permission: Permission = self
            .repo
            .create(&req.name, req.description.as_deref())
            .await?;

        Ok(permission)
    }
    async fn update(&self, id: i32, req: UpdatePermissionRequest) -> Result<Permission, AppError> {
        if let Some(name) = &req.name {
            Name::parse(name)?;
            if let Some(existing) = self.repo.find_by_name(name).await? {
                if existing.id != id {
                    return Err(AppError::Conflict("name is already registered".to_string()));
                }
            }
        }

        let permission = self
            .repo
            .update(id, req.name.as_deref(), req.description.as_deref())
            .await?;

        self.cache.delete(&cache_key(id)).await?;
        Ok(permission)
    }

    async fn delete(&self, id: i32) -> Result<(), AppError> {
        self.repo.delete(id).await?;
        self.cache.delete(&cache_key(id)).await?;
        Ok(())
    }
}
