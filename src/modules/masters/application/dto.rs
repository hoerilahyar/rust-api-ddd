use crate::modules::masters::domain::entity::{MasterGroup, MasterItem};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use validator::Validate;

// ====================================
// =========== Master Group ===========
// ====================================
#[derive(Debug, Deserialize, Validate)]
pub struct CreateMasterGroupRequest {
    #[validate(length(min = 1, max = 150, message = "code is required"))]
    pub code: String,
    #[validate(length(min = 1, max = 150, message = "name is required"))]
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateMasterGroupRequest {
    #[validate(length(min = 1, max = 150, message = "code is required"))]
    pub code: String,
    #[validate(length(min = 1, max = 150, message = "name is required"))]
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Serialize)]
pub struct MasterGroupResponse {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub items: Vec<MasterItem>,
}

impl From<MasterGroup> for MasterGroupResponse {
    fn from(u: MasterGroup) -> Self {
        Self {
            id: u.id,
            code: u.code,
            name: u.name,
            description: u.description,
            is_active: u.is_active,
            created_at: u.created_at,
            updated_at: u.updated_at,
            deleted_at: u.deleted_at,
            items: u.items,
        }
    }
}

// ====================================
// ============ Master Item ===========
// ====================================
#[derive(Debug, Deserialize, Validate)]
pub struct CreateMasterItemRequest {
    // NEW: master_items.group_id is NOT NULL — required to know which group
    // this item belongs to. If your route is nested (e.g.
    // /master-groups/{group_id}/items), you can populate this from the path
    // param in the handler instead of the body.
    #[serde(default)]
    pub group_id: i64,
    #[validate(length(min = 1, max = 150, message = "code is required"))]
    pub code: String,
    #[validate(length(min = 1, max = 150, message = "name is required"))]
    pub name: String,
    pub description: Option<String>,
    // NEW: matches master_items.extra (JSONB, defaults to '{}' in DB if omitted)
    pub extra: Option<Value>,
    // NEW: matches master_items.sort_order (defaults to 0 in DB if omitted)
    pub sort_order: Option<i32>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateMasterItemRequest {
    #[validate(length(min = 1, max = 150, message = "code is required"))]
    pub code: String,
    #[validate(length(min = 1, max = 150, message = "name is required"))]
    pub name: String,
    pub description: Option<String>,
    // NEW
    pub extra: Option<Value>,
    // NEW
    pub sort_order: Option<i32>,
    pub is_active: bool,
}

#[derive(Debug, Serialize)]
pub struct MasterItemResponse {
    pub id: i64,
    pub group_id: i64,
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

impl From<MasterItem> for MasterItemResponse {
    fn from(u: MasterItem) -> Self {
        Self {
            id: u.id,
            group_id: u.group_id,
            code: u.code,
            name: u.name,
            extra: u.extra,
            sort_order: u.sort_order,
            description: u.description,
            is_active: u.is_active,
            created_at: u.created_at,
            updated_at: u.updated_at,
            deleted_at: u.deleted_at,
        }
    }
}
