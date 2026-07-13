use async_trait::async_trait;

use crate::modules::user_setting::application::dto::UpsertUserSettingRequest;
use crate::modules::user_setting::domain::UserSetting;
use crate::shared::errors::AppError;

/// Every method is scoped to `user_id` -- handlers always pass `Claims::sub`
/// here, never a client-supplied id, so a user can only ever touch their
/// own settings.
#[async_trait]
pub trait UserSettingService: Send + Sync {
    async fn list(&self, user_id: i32) -> Result<Vec<UserSetting>, AppError>;
    async fn get(&self, user_id: i32, key: &str) -> Result<UserSetting, AppError>;
    async fn upsert(
        &self,
        user_id: i32,
        key: &str,
        req: UpsertUserSettingRequest,
    ) -> Result<UserSetting, AppError>;
    async fn delete(&self, user_id: i32, key: &str) -> Result<(), AppError>;
}
