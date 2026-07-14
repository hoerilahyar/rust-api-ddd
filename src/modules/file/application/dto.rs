use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::modules::file::domain::FileAsset;

#[derive(Debug, Serialize)]
pub struct FileResponse {
    pub uuid: Uuid,
    pub original_name: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub uploaded_by: Option<i32>,
    pub created_at: DateTime<Utc>,
}

impl From<FileAsset> for FileResponse {
    fn from(f: FileAsset) -> Self {
        Self {
            uuid: f.uuid,
            original_name: f.original_name,
            mime_type: f.mime_type,
            size_bytes: f.size_bytes,
            uploaded_by: f.uploaded_by,
            created_at: f.created_at,
        }
    }
}
