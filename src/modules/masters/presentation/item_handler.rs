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
            CreateMasterItemRequest, MasterItemResponse, UpdateMasterItemRequest,
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

pub async fn get_master_item(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    ensure_permission(&claims, "master_item.manage")?;

    let item = state.master_item_service.get_by_id(id).await?;
    Ok(ApiResponse::new("ok", MasterItemResponse::from(item)))
}

pub async fn list_master_items(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Query(pagination): Query<PaginationParams>,
) -> Result<impl IntoResponse, AppError> {
    ensure_permission(&claims, "master_item.manage")?;

    let (items, total) = state.master_item_service.list(&pagination).await?;
    let (page, limit) = pagination.normalized();
    let data: Vec<MasterItemResponse> = items.into_iter().map(MasterItemResponse::from).collect();

    Ok(PaginatedResponse::new("ok", data, page, limit, total))
}

/// Route: `POST /masters/:group_id/items`.
/// `group_id` is taken from the path, not the request body — any
/// `group_id` value in the JSON payload is overwritten here so a client
/// can never assign an item to a group it isn't targeting via the URL.
pub async fn create_master_item(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(group_id): Path<i32>,
    ValidatedJson(mut payload): ValidatedJson<CreateMasterItemRequest>,
) -> Result<impl IntoResponse, AppError> {
    ensure_permission(&claims, "master_item.manage")?;

    payload.group_id = group_id;

    let item = state
        .master_item_service
        .create(payload, claims.sub)
        .await?;
    Ok(ApiResponse::new("master item created", MasterItemResponse::from(item)).created())
}

pub async fn update_master_item(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i32>,
    ValidatedJson(payload): ValidatedJson<UpdateMasterItemRequest>,
) -> Result<impl IntoResponse, AppError> {
    ensure_permission(&claims, "master_item.manage")?;

    let item = state
        .master_item_service
        .update(id, payload, claims.sub)
        .await?;
    Ok(ApiResponse::new(
        "master item updated",
        MasterItemResponse::from(item),
    ))
}

pub async fn delete_master_item(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    ensure_permission(&claims, "master_item.manage")?;

    state.master_item_service.delete(id, claims.sub).await?;
    Ok(ApiResponse::message("master item deleted"))
}
