use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// Central application error type. Every layer (domain, application,
/// infrastructure, presentation) converges into this enum so handlers can
/// return `Result<T, AppError>` and get a consistent JSON error response.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    NotFound(String),

    #[error("{0}")]
    BadRequest(String),

    #[error("{0}")]
    Unauthorized(String),

    #[error("{0}")]
    Forbidden(String),

    #[error("{0}")]
    Conflict(String),

    #[error("validation failed")]
    Validation(Vec<FieldError>),

    #[error("too many requests")]
    TooManyRequests,

    #[error("internal server error")]
    Internal(#[from] anyhow::Error),

    #[error("database error")]
    Database(#[from] sqlx::Error),

    #[error("cache error: {0}")]
    Cache(String),
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct FieldError {
    pub field: String,
    pub message: String,
}

impl AppError {
    pub fn status(&self) -> StatusCode {
        self.status_and_message().0
    }

    pub fn message(&self) -> String {
        self.status_and_message().1
    }

    fn status_and_message(&self) -> (StatusCode, String) {
        match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, msg.clone()),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
            AppError::Validation(_) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "validation failed".to_string(),
            ),
            AppError::TooManyRequests => (
                StatusCode::TOO_MANY_REQUESTS,
                "too many requests".to_string(),
            ),
            AppError::Database(err) => {
                tracing::error!(error = ?err, "database error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error".to_string(),
                )
            }
            AppError::Cache(err) => {
                tracing::error!(error = %err, "cache error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error".to_string(),
                )
            }
            AppError::Internal(err) => {
                tracing::error!(error = ?err, "internal error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error".to_string(),
                )
            }
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        if let AppError::Validation(errors) = &self {
            let body = Json(json!({
                "success": false,
                "message": "validation failed",
                "errors": errors,
            }));
            return (StatusCode::UNPROCESSABLE_ENTITY, body).into_response();
        }

        let (status, message) = self.status_and_message();
        let body = Json(json!({
            "success": false,
            "message": message,
        }));
        (status, body).into_response()
    }
}
