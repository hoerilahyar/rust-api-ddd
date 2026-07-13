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

#[cfg(test)]
mod tests {
    use super::*;

    fn params(page: i64, limit: i64) -> PaginationParams {
        PaginationParams {
            page,
            limit,
            search: None,
        }
    }

    #[test]
    fn normalized_keeps_sane_values_as_is() {
        assert_eq!(params(2, 20).normalized(), (2, 20));
    }

    #[test]
    fn normalized_floors_page_at_one() {
        assert_eq!(params(0, 20).normalized(), (1, 20));
        assert_eq!(params(-5, 20).normalized(), (1, 20));
    }

    #[test]
    fn normalized_clamps_limit_between_one_and_hundred() {
        assert_eq!(params(1, 0).normalized(), (1, 1));
        assert_eq!(params(1, -10).normalized(), (1, 1));
        assert_eq!(params(1, 999_999).normalized(), (1, 100));
    }

    #[test]
    fn offset_matches_page_and_limit() {
        assert_eq!(params(1, 20).offset(), 0);
        assert_eq!(params(3, 20).offset(), 40);
        // Out-of-range input still normalizes before computing the offset.
        assert_eq!(params(0, 20).offset(), 0);
    }

    #[test]
    fn record_status_prioritizes_deleted_over_active_flag() {
        let now = Some(chrono::Utc::now());
        // Even an is_active=true row is "Deleted" once deleted_at is set --
        // soft-delete must win over the active flag.
        assert_eq!(RecordStatus::from_flags(true, now), RecordStatus::Deleted);
        assert_eq!(RecordStatus::from_flags(false, now), RecordStatus::Deleted);
    }

    #[test]
    fn record_status_reflects_active_flag_when_not_deleted() {
        assert_eq!(RecordStatus::from_flags(true, None), RecordStatus::Active);
        assert_eq!(RecordStatus::from_flags(false, None), RecordStatus::Inactive);
    }
}
