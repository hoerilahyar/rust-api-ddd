use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Core `Role` entity, mirrors the `role` table (+ its `role_permissions` join).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub permissions: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A single permission, as attached to a role (used both standalone and nested
/// inside `Role.Permission` responses when the caller wants ids too).
#[derive(Debug, Clone, Serialize)]
pub struct Permission {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
}
