use async_trait::async_trait;

use crate::modules::user_profile::application::dto::UpsertUserProfileRequest;
use crate::modules::user_profile::domain::UserProfile;
use crate::shared::errors::AppError;

/// Every method is scoped to `user_id`. Self-service handlers always pass
/// `Claims::sub`; the admin-facing `get` (`GET /users/:id/profile`) is the
/// only place a caller-supplied id reaches this trait, and that route is
/// gated behind `user.manage` (see `presentation::handler`).
#[async_trait]
pub trait UserProfileService: Send + Sync {
    async fn get(&self, user_id: i32) -> Result<UserProfile, AppError>;
    async fn upsert(
        &self,
        user_id: i32,
        req: UpsertUserProfileRequest,
    ) -> Result<UserProfile, AppError>;
    async fn delete(&self, user_id: i32) -> Result<(), AppError>;
}
