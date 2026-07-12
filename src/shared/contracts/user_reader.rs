use async_trait::async_trait;

use crate::shared::errors::AppError;

/// Read-only projection of a user, shared across module boundaries. The
/// `auth` module depends on this trait (not on `user`'s repository) to look
/// up credentials during login -- keeping the two modules decoupled.
#[derive(Debug, Clone)]
pub struct UserAuthProjection {
    pub id: i32,
    pub name: String,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub is_active: bool,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
}

#[async_trait]
pub trait UserReader: Send + Sync {
    /// Looks a user up by email or username (login accepts either).
    async fn find_for_auth(&self, identifier: &str) -> Result<Option<UserAuthProjection>, AppError>;

    async fn find_by_id(&self, id: i32) -> Result<Option<UserAuthProjection>, AppError>;

    async fn touch_last_login(&self, id: i32) -> Result<(), AppError>;

    /// Cross-module write used only by the password-reset flow. Kept on
    /// this contract (rather than adding a whole new trait) so `auth`
    /// doesn't need to depend on `user`'s full `UserRepository`.
    async fn update_password(&self, id: i32, password_hash: &str) -> Result<(), AppError>;
}
