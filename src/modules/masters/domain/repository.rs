use async_trait::async_trait;
use serde_json::Value;

use crate::{
    modules::masters::domain::{MasterGroup, MasterItem},
    shared::{domain::PaginationParams, errors::AppError},
};

// ====================================
// =========== Master Group ===========
// ====================================
#[async_trait]
pub trait MasterGroupRepository: Send + Sync {
    async fn find_by_id(&self, id: i64) -> Result<Option<MasterGroup>, AppError>;
    async fn find_by_name(&self, name: &str) -> Result<Option<MasterGroup>, AppError>;
    async fn find_by_code(&self, code: &str) -> Result<Option<MasterGroup>, AppError>;

    async fn list(
        &self,
        pagination: &PaginationParams,
    ) -> Result<(Vec<MasterGroup>, i64), AppError>;

    async fn create(
        &self,
        code: &str,
        name: &str,
        description: Option<&str>,
    ) -> Result<MasterGroup, AppError>;

    async fn update(
        &self,
        id: i64,
        code: Option<&str>,
        name: Option<&str>,
        description: Option<&str>,
        is_active: Option<&bool>,
    ) -> Result<MasterGroup, AppError>;

    async fn delete(&self, id: i64) -> Result<(), AppError>;
}

// ====================================
// ============ Master Item ===========
// ====================================
#[async_trait]
pub trait MasterItemRepository: Send + Sync {
    async fn find_by_id(&self, id: i64) -> Result<Option<MasterItem>, AppError>;
    async fn find_by_name(&self, name: &str) -> Result<Option<MasterItem>, AppError>;
    async fn find_by_group_and_code(
        &self,
        group_id: i64,
        code: &str,
    ) -> Result<Option<MasterItem>, AppError>;

    async fn list(&self, pagination: &PaginationParams)
        -> Result<(Vec<MasterItem>, i64), AppError>;

    async fn create(
        &self,
        group_id: i64,
        code: &str,
        name: &str,
        description: Option<&str>,
        extra: Option<Value>,
        sort_order: Option<i32>,
    ) -> Result<MasterItem, AppError>;

    async fn update(
        &self,
        id: i64,
        code: Option<&str>,
        name: Option<&str>,
        description: Option<&str>,
        extra: Option<Value>,
        sort_order: Option<i32>,
        is_active: Option<&bool>,
    ) -> Result<MasterItem, AppError>;

    async fn delete(&self, id: i64) -> Result<(), AppError>;
}
