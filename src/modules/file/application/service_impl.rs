use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use axum::body::Bytes;
use tokio::fs::File;
use uuid::Uuid;

use crate::modules::file::application::service::FileService;
use crate::modules::file::domain::{FileAsset, FileRepository, FileStorage};
use crate::shared::domain::PaginationParams;
use crate::shared::errors::AppError;

pub struct FileServiceImpl {
    repo: Arc<dyn FileRepository>,
    storage: Arc<dyn FileStorage>,
    max_upload_bytes: usize,
}

impl FileServiceImpl {
    pub fn new(repo: Arc<dyn FileRepository>, storage: Arc<dyn FileStorage>, max_upload_bytes: usize) -> Self {
        Self {
            repo,
            storage,
            max_upload_bytes,
        }
    }

    /// Keeps only the final path component of whatever the client sent as a
    /// filename, so `original_name` can never smuggle `../` or an absolute
    /// path into a response or a future path-building bug -- `stored_name`
    /// (the only thing ever used to address the disk) is always generated
    /// server-side regardless, this is purely for the display value.
    fn sanitize_original_name(raw: &str) -> String {
        let name = Path::new(raw)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("file");
        let name = if name.trim().is_empty() { "file" } else { name };
        name.chars().take(255).collect()
    }
}

#[async_trait]
impl FileService for FileServiceImpl {
    async fn upload(
        &self,
        uploaded_by: Option<i32>,
        original_name: String,
        mime_type: String,
        bytes: Bytes,
    ) -> Result<FileAsset, AppError> {
        if bytes.is_empty() {
            return Err(AppError::BadRequest("file is empty".to_string()));
        }
        if bytes.len() > self.max_upload_bytes {
            return Err(AppError::BadRequest(format!(
                "file exceeds the {} byte upload limit",
                self.max_upload_bytes
            )));
        }

        let original_name = Self::sanitize_original_name(&original_name);
        let extension = Path::new(&original_name)
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| format!(".{e}"))
            .unwrap_or_default();
        let uuid = Uuid::new_v4();
        let stored_name = format!("{uuid}{extension}");
        let mime_type = if mime_type.trim().is_empty() {
            "application/octet-stream".to_string()
        } else {
            mime_type
        };
        let size_bytes = bytes.len() as i64;

        let storage_path = self.storage.save(&stored_name, bytes).await?;

        match self
            .repo
            .create(
                uuid,
                &original_name,
                &stored_name,
                &mime_type,
                size_bytes,
                &storage_path,
                uploaded_by,
            )
            .await
        {
            Ok(file) => Ok(file),
            Err(err) => {
                // The DB row is the source of truth; if it couldn't be
                // written, don't leave an orphaned file behind on disk.
                if let Err(cleanup_err) = self.storage.delete(&storage_path).await {
                    tracing::error!(
                        error = ?cleanup_err,
                        stored_name,
                        "failed to clean up orphaned upload after a failed insert"
                    );
                }
                Err(err)
            }
        }
    }

    async fn list(&self, pagination: &PaginationParams) -> Result<(Vec<FileAsset>, i64), AppError> {
        self.repo.list(pagination).await
    }

    async fn get_by_uuid(&self, uuid: Uuid) -> Result<FileAsset, AppError> {
        self.repo
            .find_by_uuid(uuid)
            .await?
            .filter(|f| !f.is_deleted())
            .ok_or_else(|| AppError::NotFound("file not found".to_string()))
    }

    async fn open_for_download(&self, uuid: Uuid) -> Result<(FileAsset, File), AppError> {
        let file = self.get_by_uuid(uuid).await?;
        let handle = self.storage.open(&file.storage_path).await?;
        Ok((file, handle))
    }

    async fn delete(&self, uuid: Uuid) -> Result<(), AppError> {
        let file = self.get_by_uuid(uuid).await?;
        self.repo.soft_delete(file.id).await?;

        // Best-effort: the metadata row is already soft-deleted (the part
        // that matters for every other endpoint), so a disk hiccup here
        // gets logged rather than surfaced as a failed delete request.
        if let Err(err) = self.storage.delete(&file.storage_path).await {
            tracing::error!(error = ?err, file_uuid = %uuid, "failed to remove file bytes from storage");
        }

        Ok(())
    }
}
