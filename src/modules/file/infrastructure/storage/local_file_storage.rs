use std::path::{Path, PathBuf};

use async_trait::async_trait;
use axum::body::Bytes;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use crate::modules::file::domain::FileStorage;
use crate::shared::errors::AppError;

/// Stores files as plain files on local disk, under `base_path`.
///
/// `stored_name` is always a server-generated UUID (see
/// `FileServiceImpl::upload`), never the client's `original_name` --
/// so this never needs to defend against `../` in a filename it's handed.
#[derive(Clone)]
pub struct LocalFileStorage {
    base_path: PathBuf,
}

impl LocalFileStorage {
    /// Creates `base_path` (and parents) up front so the first upload
    /// doesn't race a missing directory. Synchronous (`std::fs`, not
    /// `tokio::fs`) on purpose: this runs once during `AppState::new`,
    /// which is not `async`, and a one-off directory creation at startup
    /// blocking briefly is harmless.
    pub fn new(base_path: impl Into<PathBuf>) -> anyhow::Result<Self> {
        let base_path = base_path.into();
        std::fs::create_dir_all(&base_path)?;
        Ok(Self { base_path })
    }

    fn resolve(&self, stored_name: &str) -> PathBuf {
        self.base_path.join(stored_name)
    }
}

#[async_trait]
impl FileStorage for LocalFileStorage {
    async fn save(&self, stored_name: &str, bytes: Bytes) -> Result<String, AppError> {
        let path = self.resolve(stored_name);

        let mut file = File::create(&path)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;
        file.write_all(&bytes)
            .await
            .map_err(|e| AppError::Internal(e.into()))?;
        file.flush().await.map_err(|e| AppError::Internal(e.into()))?;

        Ok(path.to_string_lossy().to_string())
    }

    async fn open(&self, storage_path: &str) -> Result<File, AppError> {
        File::open(Path::new(storage_path))
            .await
            .map_err(|e| match e.kind() {
                std::io::ErrorKind::NotFound => {
                    AppError::NotFound("file not found in storage".to_string())
                }
                _ => AppError::Internal(e.into()),
            })
    }

    async fn delete(&self, storage_path: &str) -> Result<(), AppError> {
        match tokio::fs::remove_file(storage_path).await {
            Ok(()) => Ok(()),
            // Already gone is fine -- delete is idempotent from the
            // caller's point of view.
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(AppError::Internal(e.into())),
        }
    }
}
