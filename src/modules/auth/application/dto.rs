use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    /// Accepts either the user's email or username.
    #[validate(length(min = 1, message = "email or username is required"))]
    pub identifier: String,

    #[validate(length(min = 1, message = "password is required"))]
    pub password: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct RefreshRequest {
    #[validate(length(min = 1, message = "refresh_token is required"))]
    pub refresh_token: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LogoutRequest {
    #[validate(length(min = 1, message = "refresh_token is required"))]
    pub refresh_token: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ForgotPasswordRequest {
    #[validate(email(message = "must be a valid email"))]
    pub email: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ResetPasswordRequest {
    #[validate(length(min = 1, message = "token is required"))]
    pub token: String,

    #[validate(length(min = 8, message = "password must be at least 8 characters"))]
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthUserSummary {
    pub id: i32,
    pub name: String,
    pub username: String,
    pub email: String,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct TokenPairResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: &'static str,
    pub expires_in: i64,
    pub user: AuthUserSummary,
}
