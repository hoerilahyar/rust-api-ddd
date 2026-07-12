use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Mirrors the `refresh_tokens` table. Only the hash is ever persisted --
/// the raw token is returned to the client once and never stored.
#[derive(Debug, Clone)]
pub struct RefreshToken {
    pub id: Uuid,
    pub user_id: i32,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl RefreshToken {
    pub fn is_valid(&self) -> bool {
        self.revoked_at.is_none() && self.expires_at > Utc::now()
    }
}

/// Mirrors the `password_reset_tokens` table.
#[derive(Debug, Clone)]
pub struct PasswordResetToken {
    pub id: Uuid,
    pub user_id: i32,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl PasswordResetToken {
    pub fn is_valid(&self) -> bool {
        self.used_at.is_none() && self.expires_at > Utc::now()
    }
}
