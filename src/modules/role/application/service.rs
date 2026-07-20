use async_trait::async_trait;

use crate::shared::domain::PaginationParams;
use crate::{
    modules::role::{
        application::{CreateRoleRequest, UpdateRoleRequest},
        domain::Role,
    },
    shared::errors::AppError,
};

#[async_trait]
pub trait RoleService: Send + Sync {
    async fn get_by_id(&self, id: i32) -> Result<Role, AppError>;
    async fn list(&self, pagination: &PaginationParams) -> Result<(Vec<Role>, i64), AppError>;

    async fn create(&self, req: CreateRoleRequest, actor_id: i32) -> Result<Role, AppError>;
    async fn update(&self, id: i32, req: UpdateRoleRequest, actor_id: i32) -> Result<Role, AppError>;

    async fn delete(&self, id: i32, actor_id: i32) -> Result<(), AppError>;

    /// Replaces the role's full permission set with `permission_ids` in one
    /// atomic operation: assigns ids that are missing, revokes ids that are
    /// no longer present.
    async fn sync_permissions(&self, role_id: i32, permission_ids: &[i32]) -> Result<(), AppError>;
}
