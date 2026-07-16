use async_trait::async_trait;

use crate::modules::user_profile::domain::UserProfile;
use crate::shared::errors::AppError;

/// Every method takes `user_id` explicitly and every query is scoped to it
/// -- same structural guarantee as `UserSettingRepository`: there is no
/// lookup by the profile's own `id`, so a caller can never fetch another
/// user's profile by guessing one.
#[async_trait]
pub trait UserProfileRepository: Send + Sync {
    async fn find_by_user_id(&self, user_id: i32) -> Result<Option<UserProfile>, AppError>;

    /// Create-or-replace by `user_id` (PUT semantics): fields passed as
    /// `Some` overwrite the existing value (including clearing it back to
    /// `NULL` is not supported here -- pass `None` to leave a field
    /// untouched); if no row exists yet for `user_id`, one is inserted.
    #[allow(clippy::too_many_arguments)]
    async fn upsert(
        &self,
        user_id: i32,
        phone: Option<&str>,
        address: Option<&str>,
        city: Option<&str>,
        country: Option<&str>,
        postal_code: Option<&str>,
        gender: Option<&str>,
        date_of_birth: Option<chrono::NaiveDate>,
        avatar_url: Option<&str>,
        website: Option<&str>,
        bio: Option<&str>,
    ) -> Result<UserProfile, AppError>;

    async fn delete(&self, user_id: i32) -> Result<(), AppError>;
}
