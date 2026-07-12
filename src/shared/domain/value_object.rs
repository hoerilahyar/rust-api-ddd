use serde::{Deserialize, Serialize};

/// Query params accepted by every "list" endpoint (`?page=1&limit=20&search=...`).
#[derive(Debug, Clone, Deserialize)]
pub struct PaginationParams {
    #[serde(default = "PaginationParams::default_page")]
    pub page: i64,
    #[serde(default = "PaginationParams::default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub search: Option<String>,
}

impl PaginationParams {
    fn default_page() -> i64 {
        1
    }

    fn default_limit() -> i64 {
        20
    }

    /// Clamp to sane bounds so a caller can't force `limit=999999`.
    pub fn normalized(&self) -> (i64, i64) {
        let page = self.page.max(1);
        let limit = self.limit.clamp(1, 100);
        (page, limit)
    }

    /// SQL OFFSET for the (already normalized) page/limit.
    pub fn offset(&self) -> i64 {
        let (page, limit) = self.normalized();
        (page - 1) * limit
    }
}

/// Generic non-deleted/soft-deleted marker shared by entities that support
/// soft deletes (`users`, `menus`, ...).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecordStatus {
    Active,
    Inactive,
    Deleted,
}

impl RecordStatus {
    pub fn from_flags(is_active: bool, deleted_at: Option<chrono::DateTime<chrono::Utc>>) -> Self {
        if deleted_at.is_some() {
            RecordStatus::Deleted
        } else if is_active {
            RecordStatus::Active
        } else {
            RecordStatus::Inactive
        }
    }
}
