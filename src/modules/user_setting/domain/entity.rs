use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Per-user key/value preference (theme, locale, notification prefs, ...).
/// Always scoped to a single `user_id` -- every repository/service method
/// in this module takes the owning `user_id` explicitly and every handler
/// derives it from `Claims::sub`, never from client-supplied input, so a
/// user can never read or write another user's settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSetting {
    pub id: i32,
    pub user_id: i32,
    pub key: String,
    pub value: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
