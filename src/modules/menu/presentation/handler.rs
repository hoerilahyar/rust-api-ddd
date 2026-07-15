use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Extension,
};

use crate::{
    bootstrap::state::AppState,
    modules::{
        auth::domain::Claims,
        menu::application::{
            AssignMenuPermissionRequest, CreateMenuRequest, MenuResponse, UpdateMenuRequest,
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

/// `GET /me/menu` -- any authenticated user, filtered to what their own
/// permissions allow. This is the endpoint a frontend sidebar calls.
pub async fn get_my_menu(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<impl IntoResponse, AppError> {
    let tree = state.menu_service.visible_tree(&claims.permissions).await?;
    Ok(ApiResponse::new("ok", tree))
}

/// `GET /menus/tree` -- admin, unfiltered (includes inactive menus and shows
/// every permission mapping). Requires `menu.manage`.
pub async fn get_menu_tree(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<impl IntoResponse, AppError> {
    ensure_permission(&claims, "menu.manage")?;
    let tree = state.menu_service.tree().await?;
    Ok(ApiResponse::new("ok", tree))
}

pub async fn list_menus(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Query(pagination): Query<PaginationParams>,
) -> Result<impl IntoResponse, AppError> {
    ensure_permission(&claims, "menu.manage")?;

    let (menus, total) = state.menu_service.list(&pagination).await?;
    let (page, limit) = pagination.normalized();
    let data: Vec<MenuResponse> = menus.into_iter().map(MenuResponse::from).collect();

    Ok(PaginatedResponse::new("ok", data, page, limit, total))
}

pub async fn get_menu(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    ensure_permission(&claims, "menu.manage")?;
    let menu = state.menu_service.get_by_id(id).await?;
    Ok(ApiResponse::new("ok", MenuResponse::from(menu)))
}

pub async fn create_menu(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    ValidatedJson(payload): ValidatedJson<CreateMenuRequest>,
) -> Result<impl IntoResponse, AppError> {
    ensure_permission(&claims, "menu.manage")?;
    let menu = state.menu_service.create(payload, claims.sub).await?;
    Ok(ApiResponse::new("menu created", MenuResponse::from(menu)).created())
}

pub async fn update_menu(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i32>,
    ValidatedJson(payload): ValidatedJson<UpdateMenuRequest>,
) -> Result<impl IntoResponse, AppError> {
    ensure_permission(&claims, "menu.manage")?;
    let menu = state.menu_service.update(id, payload, claims.sub).await?;
    Ok(ApiResponse::new("menu updated", MenuResponse::from(menu)))
}

pub async fn delete_menu(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i32>,
) -> Result<impl IntoResponse, AppError> {
    ensure_permission(&claims, "menu.manage")?;
    state.menu_service.delete(id, claims.sub).await?;
    Ok(ApiResponse::message("menu deleted"))
}

pub async fn assign_permission(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i32>,
    ValidatedJson(payload): ValidatedJson<AssignMenuPermissionRequest>,
) -> Result<impl IntoResponse, AppError> {
    ensure_permission(&claims, "menu.manage")?;
    state
        .menu_service
        .assign_permission(id, &payload.permission)
        .await?;
    Ok(ApiResponse::message("permission assigned"))
}

pub async fn revoke_permission(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path((id, permission)): Path<(i32, String)>,
) -> Result<impl IntoResponse, AppError> {
    ensure_permission(&claims, "menu.manage")?;
    state.menu_service.revoke_permission(id, &permission).await?;
    Ok(ApiResponse::message("permission revoked"))
}
