use async_trait::async_trait;
use redis::AsyncCommands;
use redis::aio::ConnectionManager;
use serde::{Serialize, de::DeserializeOwned};
use std::time::Duration;

use crate::shared::cache::cache_repository::CacheRepository;
use crate::shared::errors::AppError;

/// Redis-backed implementation of [`CacheRepository`] used for the
/// cache-aside pattern across domain services (user profile, menu tree, ...).
#[derive(Clone)]
pub struct RedisCacheRepository {
    conn: ConnectionManager,
}

impl RedisCacheRepository {
    pub fn new(conn: ConnectionManager) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl CacheRepository for RedisCacheRepository {
    async fn get<T: DeserializeOwned + Send>(&self, key: &str) -> Result<Option<T>, AppError> {
        let mut conn = self.conn.clone();
        let raw: Option<String> = conn
            .get(key)
            .await
            .map_err(|e| AppError::Cache(e.to_string()))?;

        match raw {
            Some(json) => {
                let value = serde_json::from_str(&json)
                    .map_err(|e| AppError::Cache(format!("deserialize failed: {e}")))?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    async fn set<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
        ttl: Duration,
    ) -> Result<(), AppError> {
        let mut conn = self.conn.clone();
        let json = serde_json::to_string(value)
            .map_err(|e| AppError::Cache(format!("serialize failed: {e}")))?;

        conn.set_ex::<_, _, ()>(key, json, ttl.as_secs().max(1))
            .await
            .map_err(|e| AppError::Cache(e.to_string()))?;
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<(), AppError> {
        let mut conn = self.conn.clone();
        conn.del::<_, ()>(key)
            .await
            .map_err(|e| AppError::Cache(e.to_string()))?;
        Ok(())
    }

    async fn delete_by_prefix(&self, prefix: &str) -> Result<(), AppError> {
        let mut conn = self.conn.clone();
        let pattern = format!("{prefix}*");
        let keys: Vec<String> = conn
            .keys(pattern)
            .await
            .map_err(|e| AppError::Cache(e.to_string()))?;

        if !keys.is_empty() {
            conn.del::<_, ()>(keys)
                .await
                .map_err(|e| AppError::Cache(e.to_string()))?;
        }
        Ok(())
    }

    async fn exists(&self, key: &str) -> Result<bool, AppError> {
        let mut conn = self.conn.clone();
        let exists: bool = conn
            .exists(key)
            .await
            .map_err(|e| AppError::Cache(e.to_string()))?;
        Ok(exists)
    }
}
