use async_trait::async_trait;

use crate::modules::masters::application::{CreateMasterItemRequest, UpdateMasterItemRequest};
use crate::modules::masters::domain::MasterItem;
use crate::shared::domain::PaginationParams;
use crate::{
    modules::masters::{
        application::{CreateMasterGroupRequest, UpdateMasterGroupRequest},
        domain::MasterGroup,
    },
    shared::errors::AppError,
};

#[async_trait]
pub trait MasterGroupService: Send + Sync {
    async fn get_by_id(&self, id: i64) -> Result<MasterGroup, AppError>;
    async fn list(
        &self,
        pagination: &PaginationParams,
    ) -> Result<(Vec<MasterGroup>, i64), AppError>;

    async fn create(
        &self,
        req: CreateMasterGroupRequest,
        actor_id: i32,
    ) -> Result<MasterGroup, AppError>;
    async fn update(
        &self,
        id: i64,
        req: UpdateMasterGroupRequest,
        actor_id: i32,
    ) -> Result<MasterGroup, AppError>;

    async fn delete(&self, id: i64, actor_id: i32) -> Result<(), AppError>;
}

#[async_trait]
pub trait MasterItemService: Send + Sync {
    async fn get_by_id(&self, id: i64) -> Result<MasterItem, AppError>;
    async fn list(&self, pagination: &PaginationParams)
        -> Result<(Vec<MasterItem>, i64), AppError>;

    async fn create(
        &self,
        req: CreateMasterItemRequest,
        actor_id: i32,
    ) -> Result<MasterItem, AppError>;
    async fn update(
        &self,
        id: i64,
        req: UpdateMasterItemRequest,
        actor_id: i32,
    ) -> Result<MasterItem, AppError>;

    async fn delete(&self, id: i64, actor_id: i32) -> Result<(), AppError>;
}
