use axum::{Json, http::StatusCode, response::IntoResponse};
use serde::Serialize;

/// Standard success envelope used by every endpoint in the API:
/// `{ "success": true, "message": "...", "data": ... }`
#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn new(message: impl Into<String>, data: T) -> Self {
        Self {
            success: true,
            message: message.into(),
            data: Some(data),
        }
    }

    /// Wraps this response with a `201 Created` status code.
    pub fn created(self) -> (StatusCode, Json<Self>) {
        (StatusCode::CREATED, Json(self))
    }
}

impl ApiResponse<()> {
    pub fn message(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
            data: None,
        }
    }
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

/// Meta block for paginated list endpoints.
#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T: Serialize> {
    pub success: bool,
    pub message: String,
    pub data: Vec<T>,
    pub meta: PaginationMeta,
}

#[derive(Debug, Serialize)]
pub struct PaginationMeta {
    pub page: i64,
    pub limit: i64,
    pub total: i64,
    pub total_pages: i64,
}

impl<T: Serialize> PaginatedResponse<T> {
    pub fn new(message: impl Into<String>, data: Vec<T>, page: i64, limit: i64, total: i64) -> Self {
        let total_pages = if limit > 0 { (total as f64 / limit as f64).ceil() as i64 } else { 0 };
        Self {
            success: true,
            message: message.into(),
            data,
            meta: PaginationMeta { page, limit, total, total_pages },
        }
    }
}

impl<T: Serialize> IntoResponse for PaginatedResponse<T> {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}
