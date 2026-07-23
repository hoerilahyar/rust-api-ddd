use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::modules::user::domain::entity::User;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(length(min = 1, max = 150, message = "name is required"))]
    pub name: String,

    #[validate(length(min = 3, max = 150, message = "username must be 3-150 characters"))]
    pub username: String,

    #[validate(email(message = "must be a valid email"))]
    pub email: String,

    #[validate(length(min = 8, message = "password must be at least 8 characters"))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUserRequest {
    #[validate(length(min = 1, max = 150, message = "name is required"))]
    pub name: Option<String>,

    #[validate(length(min = 3, max = 150, message = "username must be 3-150 characters"))]
    pub username: Option<String>,

    #[validate(email(message = "must be a valid email"))]
    pub email: Option<String>,

    #[validate(length(min = 8, message = "password must be at least 8 characters"))]
    pub password: Option<String>,

    pub is_active: Option<bool>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ChangePasswordRequest {
    #[validate(length(min = 1, message = "current password is required"))]
    pub current_password: String,

    #[validate(length(min = 8, message = "new password must be at least 8 characters"))]
    pub new_password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct AssignRoleRequest {
    #[validate(length(min = 1, message = "role name is required"))]
    pub role: String,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: i32,
    pub name: String,
    pub username: String,
    pub email: String,
    pub is_active: bool,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<User> for UserResponse {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            name: u.name,
            username: u.username,
            email: u.email,
            is_active: u.is_active,
            roles: u.roles,
            permissions: u.permissions,
            last_login_at: u.last_login_at,
            created_at: u.created_at,
            updated_at: u.updated_at,
        }
    }
}

/// Lightweight projection used by `GET /users/last-logins`. Leaves out
/// permissions/timestamps that the widget doesn't need.
#[derive(Debug, Serialize)]
pub struct LastLoginResponse {
    pub id: i32,
    pub name: String,
    pub username: String,
    pub email: String,
    pub roles: Vec<String>,
    pub last_login_at: Option<DateTime<Utc>>,
}

impl From<User> for LastLoginResponse {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            name: u.name,
            username: u.username,
            email: u.email,
            roles: u.roles,
            last_login_at: u.last_login_at,
        }
    }
}
