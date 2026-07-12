use async_trait::async_trait;

use crate::modules::user::domain::entity::User;
use crate::shared::domain::PaginationParams;
use crate::shared::errors::AppError;

/// Persistence contract for the `User` aggregate. The application layer
/// depends on this trait, not on SQLx directly (dependency inversion).
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: i32) -> Result<Option<User>, AppError>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError>;
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, AppError>;

    async fn list(&self, pagination: &PaginationParams) -> Result<(Vec<User>, i64), AppError>;

    async fn create(
        &self,
        name: &str,
        username: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<User, AppError>;

    async fn update(
        &self,
        id: i32,
        name: Option<&str>,
        email: Option<&str>,
        is_active: Option<bool>,
    ) -> Result<User, AppError>;

    async fn update_password(&self, id: i32, password_hash: &str) -> Result<(), AppError>;

    /// Soft delete: sets `deleted_at`, keeps the row for audit purposes.
    async fn soft_delete(&self, id: i32) -> Result<(), AppError>;

    async fn assign_role(&self, user_id: i32, role_id: i32, assigned_by: Option<i32>) -> Result<(), AppError>;
    async fn revoke_role(&self, user_id: i32, role_id: i32) -> Result<(), AppError>;
    async fn find_role_by_name(&self, name: &str) -> Result<Option<(i32, String)>, AppError>;
}
