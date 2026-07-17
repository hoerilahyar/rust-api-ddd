use chrono::{DateTime, Utc};
use serde::Deserialize;

/// Query params accepted by `GET /audit-trail/logs`. Deliberately separate
/// from `shared::domain::PaginationParams` (rather than composing it) since
/// axum only allows one `Query<T>` extractor per handler -- all filter
/// fields have to live on one flat struct.
#[derive(Debug, Clone, Deserialize)]
pub struct AuditTrailLogQuery {
    #[serde(default = "AuditTrailLogQuery::default_page")]
    pub page: i64,
    #[serde(default = "AuditTrailLogQuery::default_limit")]
    pub limit: i64,
    /// Matches against `ip_address` (partial, case-insensitive).
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub user_id: Option<i32>,
    #[serde(default)]
    pub action: Option<String>,
    /// Inclusive lower bound on `created_at`.
    #[serde(default)]
    pub from: Option<DateTime<Utc>>,
    /// Inclusive upper bound on `created_at`.
    #[serde(default)]
    pub to: Option<DateTime<Utc>>,
}

impl AuditTrailLogQuery {
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
