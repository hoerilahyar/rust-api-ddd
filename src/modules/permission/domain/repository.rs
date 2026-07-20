use async_trait::async_trait;

use crate::shared::domain::PaginationParams;
use crate::{modules::permission::domain::Permission, shared::errors::AppError};

/// Persistence contract for the `Permission` aggregate. The application layer
/// depends on this trait, not on SQLx directly (dependency inversion).
#[async_trait]
pub trait PermissionRepository: Send + Sync {
    async fn find_by_id(&self, id: i32) -> Result<Option<Permission>, AppError>;
    async fn find_by_name(&self, name: &str) -> Result<Option<Permission>, AppError>;

    /// Batch existence check used by role/menu permission-sync flows, so a
    /// list of `permission_id`s can be validated in one query instead of
    /// one `find_by_id` per id.
    async fn find_many_by_ids(&self, ids: &[i32]) -> Result<Vec<Permission>, AppError>;

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
