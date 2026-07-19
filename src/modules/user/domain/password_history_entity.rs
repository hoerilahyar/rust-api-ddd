use chrono::{DateTime, Utc};

/// A single historical password hash for a user, kept to prevent
/// password reuse across changes.
#[derive(Debug, Clone)]
pub struct PasswordHistory {
    pub id: i32,
    pub user_id: i32,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
}
