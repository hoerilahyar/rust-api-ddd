use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Extension,
};

use crate::{
    bootstrap::state::AppState,
    modules::{
        auth::domain::Claims,
        permission::application::{
            CreatePermissionRequest, PermissionResponse, UpdatePermissionRequest,
        },
    },
    shared::{
        domain::PaginationParams,
        errors::AppError,
        middleware::ensure_permission,
        response::{ApiResponse, PaginatedResponse},
        validator::ValidatedJson,
    },
};

pub async fn get_permission(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    ensure_permission(&claims, "permission.manage")?;

    let permission = state.permission_service.get_by_id(id).await?;
    Ok(ApiResponse::new("ok", PermissionResponse::from(permission)))
}

pub async fn list_permissions(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Query(pagination): Query<PaginationParams>,
) -> Result<impl IntoResponse, AppError> {
    ensure_permission(&claims, "permission.manage")?;

    let (permissions, total) = state.permission_service.list(&pagination).await?;
    let (page, limit) = pagination.normalized();
    let data: Vec<PermissionResponse> = permissions
        .into_iter()
        .map(PermissionResponse::from)
        .collect();

    Ok(PaginatedResponse::new("ok", data, page, limit, total))
}

pub async fn create_permission(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    ValidatedJson(payload): ValidatedJson<CreatePermissionRequest>,
) -> Result<impl IntoResponse, AppError> {
    ensure_permission(&claims, "permission.manage")?;

    let permission = state.permission_service.create(payload).await?;
    Ok(ApiResponse::new("permission created", PermissionResponse::from(permission)).created())
}

pub async fn update_permission(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i32>,
    ValidatedJson(payload): ValidatedJson<UpdatePermissionRequest>,
) -> Result<impl IntoResponse, AppError> {
    ensure_permission(&claims, "permission.manage")?;

    let permission = state.permission_service.update(id, payload).await?;
    Ok(ApiResponse::new(
        "permission updated",
        PermissionResponse::from(permission),
    ))
}

pub async fn delete_permission(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    ensure_permission(&claims, "permission.manage")?;

    state.permission_service.delete(id).await?;
    Ok(ApiResponse::message("permission deleted"))
}
