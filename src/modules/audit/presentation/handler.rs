use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Extension,
};

use crate::{
    bootstrap::state::AppState,
    modules::{audit::application::LoginLogResponse, audit::domain::LoginLogQuery, auth::domain::Claims},
    shared::{
        errors::AppError,
        middleware::ensure_permission,
        response::{ApiResponse, PaginatedResponse},
    },
};

pub async fn list_login_logs(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<LoginLogQuery>,
) -> Result<impl IntoResponse, AppError> {
    ensure_permission(&claims, "audit.read")?;

    let (logs, total) = state.audit_log_service.list(&query).await?;
    let (page, limit) = query.normalized();
    let data: Vec<LoginLogResponse> = logs.into_iter().map(LoginLogResponse::from).collect();

    Ok(PaginatedResponse::new("ok", data, page, limit, total))
}

pub async fn get_login_log(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    ensure_permission(&claims, "audit.read")?;

    let log = state.audit_log_service.get_by_id(id).await?;
    Ok(ApiResponse::new("ok", LoginLogResponse::from(log)))
}
