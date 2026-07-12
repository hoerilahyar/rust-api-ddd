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

    async fn create(&self, req: CreateRoleRequest) -> Result<Role, AppError>;
    async fn update(&self, id: i32, req: UpdateRoleRequest) -> Result<Role, AppError>;

    async fn delete(&self, id: i32) -> Result<(), AppError>;

    async fn assign_permission(&self, role_id: i32, permission_name: &str) -> Result<(), AppError>;
    async fn revoke_permission(&self, role_id: i32, permission_name: &str) -> Result<(), AppError>;
}
