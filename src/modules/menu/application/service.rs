use async_trait::async_trait;

use crate::modules::menu::application::dto::{CreateMenuRequest, MenuTreeNode, UpdateMenuRequest};
use crate::modules::menu::domain::Menu;
use crate::shared::domain::PaginationParams;
use crate::shared::errors::AppError;

#[async_trait]
pub trait MenuService: Send + Sync {
    async fn get_by_id(&self, id: i32) -> Result<Menu, AppError>;
    async fn list(&self, pagination: &PaginationParams) -> Result<(Vec<Menu>, i64), AppError>;

    /// Full tree, every menu regardless of `is_active` or permission mapping.
    async fn tree(&self) -> Result<Vec<MenuTreeNode>, AppError>;

    /// Tree filtered down to what a caller holding `user_permissions` is
    /// allowed to see (and only active menus).
    async fn visible_tree(&self, user_permissions: &[String]) -> Result<Vec<MenuTreeNode>, AppError>;

    async fn create(&self, req: CreateMenuRequest, actor_id: i32) -> Result<Menu, AppError>;
    async fn update(&self, id: i32, req: UpdateMenuRequest, actor_id: i32) -> Result<Menu, AppError>;
    async fn delete(&self, id: i32, actor_id: i32) -> Result<(), AppError>;

    async fn assign_permission(&self, menu_id: i32, permission_name: &str) -> Result<(), AppError>;
    async fn revoke_permission(&self, menu_id: i32, permission_name: &str) -> Result<(), AppError>;
}
