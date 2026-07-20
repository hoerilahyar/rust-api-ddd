use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Extension,
};

use crate::{
    bootstrap::state::AppState,
    modules::{
        auth::domain::Claims,
        role::application::{
            CreateRoleRequest, RoleResponse, SyncRolePermissionsRequest, UpdateRoleRequest,
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

pub async fn get_role(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    ensure_permission(&claims, "role.manage")?;

    let role = state.role_service.get_by_id(id).await?;
    Ok(ApiResponse::new("ok", RoleResponse::from(role)))
}

pub async fn list_roles(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Query(pagination): Query<PaginationParams>,
) -> Result<impl IntoResponse, AppError> {
    ensure_permission(&claims, "role.manage")?;

    let (roles, total) = state.role_service.list(&pagination).await?;
    let (page, limit) = pagination.normalized();
    let data: Vec<RoleResponse> = roles.into_iter().map(RoleResponse::from).collect();

    Ok(PaginatedResponse::new("ok", data, page, limit, total))
}

pub async fn create_role(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    ValidatedJson(payload): ValidatedJson<CreateRoleRequest>,
) -> Result<impl IntoResponse, AppError> {
    ensure_permission(&claims, "role.manage")?;

    let role = state.role_service.create(payload, claims.sub).await?;
    Ok(ApiResponse::new("role created", RoleResponse::from(role)).created())
}

pub async fn update_role(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i32>,
    ValidatedJson(payload): ValidatedJson<UpdateRoleRequest>,
) -> Result<impl IntoResponse, AppError> {
    ensure_permission(&claims, "role.manage")?;

    let role = state.role_service.update(id, payload, claims.sub).await?;
    Ok(ApiResponse::new("role updated", RoleResponse::from(role)))
}

pub async fn delete_role(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    ensure_permission(&claims, "role.manage")?;

    state.role_service.delete(id, claims.sub).await?;
    Ok(ApiResponse::message("role deleted"))
}

pub async fn sync_permissions(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i32>,
    ValidatedJson(payload): ValidatedJson<SyncRolePermissionsRequest>,
) -> Result<impl IntoResponse, AppError> {
    ensure_permission(&claims, "role.manage")?;

    state
        .role_service
        .sync_permissions(id, &payload.permission_ids)
        .await?;

    Ok(ApiResponse::message("permissions synced"))
}
