use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

/// Metadata row for one uploaded file. The bytes themselves live wherever
/// `FileStorage` put them (`storage_path`); this entity never holds the
/// bytes in memory.
#[derive(Debug, Clone, Serialize)]
pub struct FileAsset {
    pub id: i64,
    pub uuid: Uuid,
    pub original_name: String,
    pub stored_name: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub storage_path: String,
    pub uploaded_by: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl FileAsset {
    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }
}
