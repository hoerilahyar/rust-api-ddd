use async_trait::async_trait;

use crate::shared::domain::PaginationParams;
use crate::{modules::role::domain::Role, shared::errors::AppError};

/// Persistence contract for the `Role` aggregate. The application layer
/// depends on this trait, not on SQLx directly (dependency inversion).
#[async_trait]
pub trait RoleRepository: Send + Sync {
    async fn find_by_id(&self, id: i32) -> Result<Option<Role>, AppError>;
    async fn find_by_name(&self, name: &str) -> Result<Option<Role>, AppError>;

    async fn list(&self, pagination: &PaginationParams) -> Result<(Vec<Role>, i64), AppError>;

    async fn create(&self, name: &str, description: Option<&str>) -> Result<Role, AppError>;

    async fn update(
        &self,
        id: i32,
        name: Option<&str>,
        description: Option<&str>,
    ) -> Result<Role, AppError>;

    async fn delete(&self, id: i32) -> Result<(), AppError>;

    async fn assign_permission(&self, role_id: i32, permission_id: i32) -> Result<(), AppError>;
    async fn revoke_permission(&self, role_id: i32, permission_id: i32) -> Result<(), AppError>;
    async fn find_permission_by_name(&self, name: &str) -> Result<Option<(i32, String)>, AppError>;
}
