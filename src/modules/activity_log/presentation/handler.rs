use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Extension,
};

use crate::{
    bootstrap::state::AppState,
    modules::{
        activity_log::application::ActivityLogResponse, activity_log::domain::ActivityLogQuery,
        auth::domain::Claims,
    },
    shared::{
        errors::AppError,
        middleware::ensure_permission,
        response::{ApiResponse, PaginatedResponse},
    },
};

pub async fn list_activity_logs(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<ActivityLogQuery>,
) -> Result<impl IntoResponse, AppError> {
    ensure_permission(&claims, "activity_log.read")?;

    let (logs, total) = state.activity_log_service.list(&query).await?;
    let (page, limit) = query.normalized();
    let data: Vec<ActivityLogResponse> = logs.into_iter().map(ActivityLogResponse::from).collect();

    Ok(PaginatedResponse::new("ok", data, page, limit, total))
}

pub async fn get_activity_log(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    ensure_permission(&claims, "activity_log.read")?;

    let log = state.activity_log_service.get_by_id(id).await?;
    Ok(ApiResponse::new("ok", ActivityLogResponse::from(log)))
}
