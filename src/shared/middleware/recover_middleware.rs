use axum::{Json, http::StatusCode, response::IntoResponse};
use serde_json::json;
use std::any::Any;
use tower_http::catch_panic::CatchPanicLayer;

/// Wraps the whole router with panic recovery: a panicking handler returns
/// a JSON 500 instead of silently killing the connection / whole server.
pub fn recover_layer() -> CatchPanicLayer<fn(Box<dyn Any + Send>) -> axum::response::Response> {
    CatchPanicLayer::custom(handle_panic)
}

fn handle_panic(err: Box<dyn Any + Send>) -> axum::response::Response {
    let message = if let Some(s) = err.downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = err.downcast_ref::<String>() {
        s.clone()
    } else {
        "unexpected panic".to_string()
    };

    tracing::error!(panic = %message, "handler panicked");

    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({
            "success": false,
            "message": "internal server error",
        })),
    )
        .into_response()
}

