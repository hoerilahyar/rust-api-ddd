use async_trait::async_trait;
use axum::body::Bytes;
use tokio::fs::File;
use uuid::Uuid;

use crate::modules::file::domain::FileAsset;
use crate::shared::domain::PaginationParams;
use crate::shared::errors::AppError;

#[async_trait]
pub trait FileService: Send + Sync {
    async fn upload(
        &self,
        uploaded_by: Option<i32>,
        original_name: String,
        mime_type: String,
        bytes: Bytes,
        actor_id: i32,
    ) -> Result<FileAsset, AppError>;

    async fn list(&self, pagination: &PaginationParams) -> Result<(Vec<FileAsset>, i64), AppError>;

    async fn get_by_uuid(&self, uuid: Uuid) -> Result<FileAsset, AppError>;

    /// Returns the metadata plus an open handle to the bytes, ready to be
    /// streamed straight into the HTTP response body.
    async fn open_for_download(
        &self,
        uuid: Uuid,
        actor_id: i32,
    ) -> Result<(FileAsset, File), AppError>;

    async fn delete(&self, uuid: Uuid, actor_id: i32) -> Result<(), AppError>;
}
