use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Core `Permission` entity, mirrors the `role` table (+ its `role_permissions` join).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}
