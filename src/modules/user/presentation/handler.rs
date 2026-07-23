use axum::extract::{Extension, Path, Query, State};
use axum::response::IntoResponse;
use serde::Deserialize;

use crate::bootstrap::state::AppState;
use crate::modules::auth::domain::value_object::Claims;
use crate::modules::user::application::dto::{
    AssignRoleRequest, ChangePasswordRequest, CreateUserRequest, LastLoginResponse,
    UpdateUserRequest, UserResponse,
};
use crate::shared::domain::PaginationParams;
use crate::shared::errors::AppError;
use crate::shared::middleware::ensure_permission;
use crate::shared::response::{ApiResponse, PaginatedResponse};
use crate::shared::validator::ValidatedJson;

pub async fn me(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<impl IntoResponse, AppError> {
    let user = state.user_service.get_by_id(claims.sub).await?;
    Ok(ApiResponse::new("ok", UserResponse::from(user)))
}

pub async fn list_users(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Query(pagination): Query<PaginationParams>,
) -> Result<impl IntoResponse, AppError> {
    ensure_permission(&claims, "user.manage")?;

    let (users, total) = state.user_service.list(&pagination).await?;
    let (page, limit) = pagination.normalized();
    let data: Vec<UserResponse> = users.into_iter().map(UserResponse::from).collect();

    Ok(PaginatedResponse::new("ok", data, page, limit, total))
}

const DEFAULT_LAST_LOGINS_LIMIT: i64 = 10;
const MAX_LAST_LOGINS_LIMIT: i64 = 50;

#[derive(Debug, Deserialize)]
pub struct LastLoginsQuery {
    pub limit: Option<i64>,
}

/// `GET /users/last-logins?limit=10` — most recently logged-in users,
/// newest first. Used by the dashboard "recent logins" widget.
pub async fn list_last_logins(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<LastLoginsQuery>,
) -> Result<impl IntoResponse, AppError> {
    ensure_permission(&claims, "user.manage")?;

    let limit = query
        .limit
        .unwrap_or(DEFAULT_LAST_LOGINS_LIMIT)
        .clamp(1, MAX_LAST_LOGINS_LIMIT);

    let users = state.user_service.list_last_logins(limit).await?;
    let data: Vec<LastLoginResponse> = users.into_iter().map(LastLoginResponse::from).collect();

    Ok(ApiResponse::new("ok", data))
}

pub async fn get_user(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    ensure_permission(&claims, "user.manage")?;

    let user = state.user_service.get_by_id(id).await?;
    Ok(ApiResponse::new("ok", UserResponse::from(user)))
}

pub async fn create_user(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    ValidatedJson(payload): ValidatedJson<CreateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    ensure_permission(&claims, "user.manage")?;

    let user = state.user_service.create(payload, claims.sub).await?;
    Ok(ApiResponse::new("user created", UserResponse::from(user)).created())
}

pub async fn update_user(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i32>,
    ValidatedJson(payload): ValidatedJson<UpdateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    ensure_permission(&claims, "user.manage")?;

    let user = state.user_service.update(id, payload, claims.sub).await?;
    Ok(ApiResponse::new("user updated", UserResponse::from(user)))
}

pub async fn change_my_password(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    ValidatedJson(payload): ValidatedJson<ChangePasswordRequest>,
) -> Result<impl IntoResponse, AppError> {
    state
        .user_service
        .change_password(
            claims.sub,
            &payload.current_password,
            &payload.new_password,
            claims.sub,
        )
        .await?;

    Ok(ApiResponse::message("password changed"))
}

pub async fn delete_user(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    ensure_permission(&claims, "user.manage")?;

    state.user_service.delete(id, claims.sub).await?;
    Ok(ApiResponse::message("user deleted"))
}

pub async fn assign_role(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i32>,
    ValidatedJson(payload): ValidatedJson<AssignRoleRequest>,
) -> Result<impl IntoResponse, AppError> {
    ensure_permission(&claims, "user.manage")?;

    state
        .user_service
        .assign_role(id, &payload.role, Some(claims.sub), claims.sub)
        .await?;

    Ok(ApiResponse::message("role assigned"))
}

pub async fn revoke_role(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path((id, role_id)): Path<(i32, String)>,
) -> Result<impl IntoResponse, AppError> {
    ensure_permission(&claims, "user.manage")?;

    state
        .user_service
        .revoke_role(id, &role_id, claims.sub)
        .await?;
    Ok(ApiResponse::message("role revoked"))
}
