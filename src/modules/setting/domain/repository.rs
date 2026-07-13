use async_trait::async_trait;

use crate::modules::setting::domain::SystemSetting;
use crate::shared::errors::AppError;

#[async_trait]
pub trait SettingRepository: Send + Sync {
    async fn list_all(&self) -> Result<Vec<SystemSetting>, AppError>;
    async fn find_by_key(&self, key: &str) -> Result<Option<SystemSetting>, AppError>;

    /// Create-or-replace by key (PUT semantics): if `key` already exists,
    /// `value`/`updated_by` are overwritten (a `None` value clears it);
    /// `description` is only overwritten when `Some` is passed, otherwise
    /// the existing description is kept. If `key` doesn't exist yet, a new
    /// row is inserted.
    async fn upsert(
        &self,
        key: &str,
        value: Option<&str>,
        description: Option<&str>,
        updated_by: Option<i32>,
    ) -> Result<SystemSetting, AppError>;

    async fn delete(&self, key: &str) -> Result<(), AppError>;
}
