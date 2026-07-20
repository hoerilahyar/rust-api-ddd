use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::modules::role::domain::Role;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateRoleRequest {
    #[validate(length(min = 1, max = 150, message = "name is required"))]
    pub name: String,

    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateRoleRequest {
    #[validate(length(min = 1, max = 150, message = "name is required"))]
    pub name: Option<String>,

    pub description: Option<String>,

    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct SyncRolePermissionsRequest {
    /// Full desired permission-id list for this role; the server diffs
    /// this against what's currently assigned and adds/removes accordingly.
    /// An empty list revokes every permission from the role.
    pub permission_ids: Vec<i32>,
}

#[derive(Debug, Serialize)]
pub struct RoleResponse {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub permissions: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Role> for RoleResponse {
    fn from(u: Role) -> Self {
        Self {
            id: u.id,
            name: u.name,
            description: u.description,
            permissions: u.permissions,
            created_at: u.created_at,
            updated_at: u.updated_at,
        }
    }
}
