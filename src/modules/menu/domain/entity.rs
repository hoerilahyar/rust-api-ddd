use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Core `Menu` entity, mirrors the `menus` table (+ its `menu_permissions`
/// join). A menu with an empty `permissions` list is visible to any
/// authenticated user (e.g. "Dashboard"); otherwise the caller needs at
/// least one of the listed permissions to see it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Menu {
    pub id: i32,
    pub parent_id: Option<i32>,
    pub name: String,
    pub slug: String,
    pub path: Option<String>,
    pub icon: Option<String>,
    pub order_index: i32,
    pub is_active: bool,
    #[serde(default)]
    pub permissions: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
