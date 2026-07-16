use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Extension,
};

use crate::{
    bootstrap::state::AppState,
    modules::{
        auth::domain::Claims,
        masters::application::{
            CreateMasterGroupRequest, MasterGroupResponse, UpdateMasterGroupRequest,
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

pub async fn get_master_group(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    ensure_permission(&claims, "masters.manage")?;

    let group = state.master_group_service.get_by_id(id).await?;
    Ok(ApiResponse::new("ok", MasterGroupResponse::from(group)))
}

pub async fn list_master_groups(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Query(pagination): Query<PaginationParams>,
) -> Result<impl IntoResponse, AppError> {
    ensure_permission(&claims, "masters.manage")?;

    let (groups, total) = state.master_group_service.list(&pagination).await?;
    let (page, limit) = pagination.normalized();
    let data: Vec<MasterGroupResponse> =
        groups.into_iter().map(MasterGroupResponse::from).collect();

    Ok(PaginatedResponse::new("ok", data, page, limit, total))
}

pub async fn create_master_group(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    ValidatedJson(payload): ValidatedJson<CreateMasterGroupRequest>,
) -> Result<impl IntoResponse, AppError> {
    ensure_permission(&claims, "masters.manage")?;

    let group = state
        .master_group_service
        .create(payload, claims.sub)
        .await?;
    Ok(ApiResponse::new("master group created", MasterGroupResponse::from(group)).created())
}

pub async fn update_master_group(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
    ValidatedJson(payload): ValidatedJson<UpdateMasterGroupRequest>,
) -> Result<impl IntoResponse, AppError> {
    ensure_permission(&claims, "masters.manage")?;

    let group = state
        .master_group_service
        .update(id, payload, claims.sub)
        .await?;
    Ok(ApiResponse::new(
        "master group updated",
        MasterGroupResponse::from(group),
    ))
}

pub async fn delete_master_group(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    ensure_permission(&claims, "masters.manage")?;

    state.master_group_service.delete(id, claims.sub).await?;
    Ok(ApiResponse::message("master group deleted"))
}
