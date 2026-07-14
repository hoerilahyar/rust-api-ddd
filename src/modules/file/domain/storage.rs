use async_trait::async_trait;
use axum::body::Bytes;
use tokio::fs::File;

use crate::shared::errors::AppError;

/// Port for wherever file bytes actually live. `FileRepository` only ever
/// stores metadata (see `entity::FileAsset`); this trait is the only thing
/// that touches real bytes, so swapping local-disk storage for S3 (or
/// anything else) later means writing one new implementation, no changes
/// to the domain/application layers.
#[async_trait]
pub trait FileStorage: Send + Sync {
    /// Persists `bytes` under `stored_name` and returns the
    /// implementation-specific path/key to record as `storage_path`.
    async fn save(&self, stored_name: &str, bytes: Bytes) -> Result<String, AppError>;

    /// Opens `storage_path` for streaming back to the client.
    async fn open(&self, storage_path: &str) -> Result<File, AppError>;

    /// Best-effort physical delete. Callers should not fail the whole
    /// delete operation just because this errors (e.g. file already
    /// missing) -- log and move on.
    async fn delete(&self, storage_path: &str) -> Result<(), AppError>;
}
