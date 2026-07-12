use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::modules::permission::domain::Permission;

#[derive(Debug, Deserialize, Validate)]
pub struct CreatePermissionRequest {
    #[validate(length(min = 1, max = 150, message = "name is required"))]
    pub name: String,

    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdatePermissionRequest {
    #[validate(length(min = 1, max = 150, message = "name is required"))]
    pub name: Option<String>,

    pub description: Option<String>,

    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct PermissionResponse {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<Permission> for PermissionResponse {
    fn from(u: Permission) -> Self {
        Self {
            id: u.id,
            name: u.name,
            description: u.description,
            created_at: u.created_at,
        }
    }
}
