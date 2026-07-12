use async_trait::async_trait;
use serde::{Serialize, de::DeserializeOwned};
use std::time::Duration;

use crate::shared::errors::AppError;

/// Cache-aside abstraction so application services depend on a trait, not
/// on Redis directly. Any backend (Redis, in-memory, ...) can implement this.
#[async_trait]
pub trait CacheRepository: Send + Sync {
    async fn get<T: DeserializeOwned + Send>(&self, key: &str) -> Result<Option<T>, AppError>;

    async fn set<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
        ttl: Duration,
    ) -> Result<(), AppError>;

    async fn delete(&self, key: &str) -> Result<(), AppError>;

    /// Delete every key matching a `prefix*` pattern (used to bust list
    /// caches like `user:list:*` after a write).
    async fn delete_by_prefix(&self, prefix: &str) -> Result<(), AppError>;

    async fn exists(&self, key: &str) -> Result<bool, AppError>;
}
