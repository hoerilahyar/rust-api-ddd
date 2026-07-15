use async_trait::async_trait;

use crate::modules::setting::application::dto::UpsertSettingRequest;
use crate::modules::setting::domain::SystemSetting;
use crate::shared::errors::AppError;

#[async_trait]
pub trait SettingService: Send + Sync {
    async fn list(&self) -> Result<Vec<SystemSetting>, AppError>;
    async fn get_by_key(&self, key: &str) -> Result<SystemSetting, AppError>;

    /// Create-or-replace by key. `updated_by` is the caller's own user id
    /// (from `Claims::sub`), recorded for audit purposes.
    async fn upsert(
        &self,
        key: &str,
        req: UpsertSettingRequest,
        updated_by: i32,
    ) -> Result<SystemSetting, AppError>;

    async fn delete(&self, key: &str, actor_id: i32) -> Result<(), AppError>;
}
