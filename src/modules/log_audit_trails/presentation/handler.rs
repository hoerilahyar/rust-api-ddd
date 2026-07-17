use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Extension,
};

use crate::{
    bootstrap::state::AppState,
    modules::{
        auth::domain::Claims,
        log_audit_trails::{application::AuditTrailLogResponse, domain::AuditTrailLogQuery},
    },
    shared::{
        errors::AppError,
        middleware::ensure_permission,
        response::{ApiResponse, PaginatedResponse},
    },
};

pub async fn list_audit_trail_logs(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<AuditTrailLogQuery>,
) -> Result<impl IntoResponse, AppError> {
    ensure_permission(&claims, "log_audit_trail.read")?;

    let (logs, total) = state.audit_trail_log_service.list(&query).await?;
    let (page, limit) = query.normalized();
    let data: Vec<AuditTrailLogResponse> =
        logs.into_iter().map(AuditTrailLogResponse::from).collect();

    Ok(PaginatedResponse::new("ok", data, page, limit, total))
}

pub async fn get_audit_trail_log(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    ensure_permission(&claims, "log_audit_trail.read")?;

    let log = state.audit_trail_log_service.get_by_id(id).await?;
    Ok(ApiResponse::new("ok", AuditTrailLogResponse::from(log)))
}
