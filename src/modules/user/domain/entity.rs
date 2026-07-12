use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Core `User` entity, mirrors the `users` table (+ its `user_roles` join).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub is_active: bool,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub roles: Vec<String>,
}

impl User {
    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }
}

/// A single role, as attached to a user (used both standalone and nested
/// inside `User.roles` responses when the caller wants ids too).
#[derive(Debug, Clone, Serialize)]
pub struct Role {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
}
