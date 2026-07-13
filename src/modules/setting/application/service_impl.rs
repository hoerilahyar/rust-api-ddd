use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;

use crate::modules::setting::application::dto::UpsertSettingRequest;
use crate::modules::setting::application::service::SettingService;
use crate::modules::setting::domain::{SettingDomainError, SettingRepository, SystemSetting};
use crate::shared::cache::{CacheRepository, RedisCacheRepository};
use crate::shared::errors::AppError;

const CACHE_TTL: Duration = Duration::from_secs(300);
const LIST_CACHE_KEY: &str = "setting:list";

fn cache_key(key: &str) -> String {
    format!("setting:key:{key}")
}

pub struct SettingServiceImpl {
    repo: Arc<dyn SettingRepository>,
    cache: Arc<RedisCacheRepository>,
}

impl SettingServiceImpl {
    pub fn new(repo: Arc<dyn SettingRepository>, cache: Arc<RedisCacheRepository>) -> Self {
        Self { repo, cache }
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

        let setting = self
            .repo
            .upsert(
                key,
                req.value.as_deref(),
                req.description.as_deref(),
                Some(updated_by),
            )
            .await?;

        self.invalidate(key).await?;
        Ok(setting)
    }

    async fn delete(&self, key: &str) -> Result<(), AppError> {
        self.repo.delete(key).await?;
        self.invalidate(key).await?;
        Ok(())
    }
}
