use async_trait::async_trait;

use crate::shared::domain::PaginationParams;
use crate::{
    modules::permission::{
        application::{CreatePermissionRequest, UpdatePermissionRequest},
        domain::Permission,
    },
    shared::errors::AppError,
};

#[async_trait]
pub trait PermissionService: Send + Sync {
    async fn get_by_id(&self, id: i32) -> Result<Permission, AppError>;
    async fn list(&self, pagination: &PaginationParams)
        -> Result<(Vec<Permission>, i64), AppError>;

    async fn create(&self, req: CreatePermissionRequest) -> Result<Permission, AppError>;
    async fn update(&self, id: i32, req: UpdatePermissionRequest) -> Result<Permission, AppError>;

    async fn delete(&self, id: i32) -> Result<(), AppError>;
}
