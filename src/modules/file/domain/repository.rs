use async_trait::async_trait;
use uuid::Uuid;

use crate::modules::file::domain::FileAsset;
use crate::shared::domain::PaginationParams;
use crate::shared::errors::AppError;

#[async_trait]
pub trait FileRepository: Send + Sync {
    #[allow(clippy::too_many_arguments)]
    async fn create(
        &self,
        uuid: Uuid,
        original_name: &str,
        stored_name: &str,
        mime_type: &str,
        size_bytes: i64,
        storage_path: &str,
        uploaded_by: Option<i32>,
    ) -> Result<FileAsset, AppError>;

    async fn find_by_id(&self, id: i64) -> Result<Option<FileAsset>, AppError>;

    async fn find_by_uuid(&self, uuid: Uuid) -> Result<Option<FileAsset>, AppError>;

    async fn list(&self, pagination: &PaginationParams) -> Result<(Vec<FileAsset>, i64), AppError>;

    /// Soft-delete only (`deleted_at = now()`). Removing the physical bytes
    /// from storage is the service's job (via `FileStorage`), kept separate
    /// so a failed disk delete never leaves a half-updated DB row.
    async fn soft_delete(&self, id: i64) -> Result<(), AppError>;
}
