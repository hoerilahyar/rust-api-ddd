use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Core `SystemSetting` entity, mirrors the `system_settings` table -- a
/// generic admin-configurable key/value store (upload limits, default
/// storage provider, pagination defaults, etc.). Global, not per-user; see
/// `modules::user_setting` for the per-user equivalent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemSetting {
    pub id: i32,
    pub key: String,
    pub value: Option<String>,
    pub description: Option<String>,
    pub updated_by: Option<i32>,
    pub updated_at: DateTime<Utc>,
}
