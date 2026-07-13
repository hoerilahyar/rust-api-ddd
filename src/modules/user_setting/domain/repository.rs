use async_trait::async_trait;

use crate::modules::user_setting::domain::UserSetting;
use crate::shared::errors::AppError;

/// Every method takes `user_id` explicitly and every query is scoped to it
/// (`WHERE user_id = ...`) -- there is no `find_by_id`-only lookup here on
/// purpose, so it's structurally impossible for a caller to fetch another
/// user's row by guessing an id.
#[async_trait]
pub trait UserSettingRepository: Send + Sync {
    async fn list_for_user(&self, user_id: i32) -> Result<Vec<UserSetting>, AppError>;
    async fn find(&self, user_id: i32, key: &str) -> Result<Option<UserSetting>, AppError>;

    /// Create-or-replace by `(user_id, key)`.
    async fn upsert(
        &self,
        user_id: i32,
        key: &str,
        value: Option<&str>,
    ) -> Result<UserSetting, AppError>;

    async fn delete(&self, user_id: i32, key: &str) -> Result<(), AppError>;
}
