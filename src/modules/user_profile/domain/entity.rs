use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

/// Extended profile data for a user (contact info, bio, avatar, ...),
/// mirrors the `user_profiles` table. Deliberately kept out of `users`
/// (see `modules::user`) so the core identity/auth row stays small and
/// this can grow (more fields, versioning, etc.) without touching it.
///
/// Strictly 1:1 with `User`: every row carries a unique `user_id` and every
/// repository/service method in this module takes that `user_id` explicitly,
/// with handlers deriving it from `Claims::sub` for the self-service routes
/// (see `modules::user_setting` for the same pattern).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: i32,
    pub user_id: i32,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub postal_code: Option<String>,
    pub gender: Option<String>,
    pub date_of_birth: Option<NaiveDate>,
    pub avatar_url: Option<String>,
    pub website: Option<String>,
    pub bio: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
