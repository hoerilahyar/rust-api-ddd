use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;

use crate::modules::user_setting::application::dto::UpsertUserSettingRequest;
use crate::modules::user_setting::application::service::UserSettingService;
use crate::modules::user_setting::domain::{
    UserSetting, UserSettingDomainError, UserSettingRepository,
};
use crate::shared::cache::{CacheRepository, RedisCacheRepository};
use crate::shared::errors::AppError;

const CACHE_TTL: Duration = Duration::from_secs(300);

fn list_cache_key(user_id: i32) -> String {
    format!("user_setting:list:{user_id}")
}

fn item_cache_key(user_id: i32, key: &str) -> String {
    format!("user_setting:item:{user_id}:{key}")
}

pub struct UserSettingServiceImpl {
    repo: Arc<dyn UserSettingRepository>,
    cache: Arc<RedisCacheRepository>,
}

impl UserSettingServiceImpl {
    pub fn new(repo: Arc<dyn UserSettingRepository>, cache: Arc<RedisCacheRepository>) -> Self {
        Self { repo, cache }
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

        let setting = self.repo.upsert(user_id, key, req.value.as_deref()).await?;
        self.invalidate(user_id, key).await?;
        Ok(setting)
    }

    async fn delete(&self, user_id: i32, key: &str) -> Result<(), AppError> {
        self.repo.delete(user_id, key).await?;
        self.invalidate(user_id, key).await?;
        Ok(())
    }
}
