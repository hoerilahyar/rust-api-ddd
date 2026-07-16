use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

// ====================================
// =========== Master Group ===========
// ====================================
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterGroup {
    pub id: i32,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub items: Vec<String>,
}

// ====================================
// ============ Master Item ===========
// ====================================
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasterItem {
    pub id: i32,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub extra: Option<Value>,
    pub sort_order: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}
