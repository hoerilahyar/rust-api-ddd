use async_trait::async_trait;

use crate::shared::domain::PaginationParams;
use crate::{modules::permission::domain::Permission, shared::errors::AppError};

/// Persistence contract for the `Permission` aggregate. The application layer
/// depends on this trait, not on SQLx directly (dependency inversion).
#[async_trait]
pub trait PermissionRepository: Send + Sync {
    async fn find_by_id(&self, id: i32) -> Result<Option<Permission>, AppError>;
    async fn find_by_name(&self, name: &str) -> Result<Option<Permission>, AppError>;

    async fn list(&self, pagination: &PaginationParams)
        -> Result<(Vec<Permission>, i64), AppError>;

    async fn create(&self, name: &str, description: Option<&str>) -> Result<Permission, AppError>;

    async fn update(
        &self,
        id: i32,
        name: Option<&str>,
        description: Option<&str>,
    ) -> Result<Permission, AppError>;

    async fn delete(&self, id: i32) -> Result<(), AppError>;
}
