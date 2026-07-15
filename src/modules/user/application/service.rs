use async_trait::async_trait;

use crate::modules::user::application::dto::{CreateUserRequest, UpdateUserRequest};
use crate::modules::user::domain::entity::User;
use crate::shared::domain::PaginationParams;
use crate::shared::errors::AppError;

#[async_trait]
pub trait UserService: Send + Sync {
    async fn get_by_id(&self, id: i32) -> Result<User, AppError>;
    async fn list(&self, pagination: &PaginationParams) -> Result<(Vec<User>, i64), AppError>;
    async fn create(&self, req: CreateUserRequest, actor_id: i32) -> Result<User, AppError>;
    async fn update(
        &self,
        id: i32,
        req: UpdateUserRequest,
        actor_id: i32,
    ) -> Result<User, AppError>;
    async fn change_password(
        &self,
        id: i32,
        current_password: &str,
        new_password: &str,
        actor_id: i32,
    ) -> Result<(), AppError>;
    async fn delete(&self, id: i32, actor_id: i32) -> Result<(), AppError>;
    async fn assign_role(
        &self,
        user_id: i32,
        role_name: &str,
        assigned_by: Option<i32>,
        actor_id: i32,
    ) -> Result<(), AppError>;
    async fn revoke_role(
        &self,
        user_id: i32,
        role_name: &str,
        actor_id: i32,
    ) -> Result<(), AppError>;
}
