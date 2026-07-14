use axum::extract::DefaultBodyLimit;
use axum::routing::{delete, get, post};
use axum::Router;

use crate::bootstrap::state::AppState;
use crate::modules::file::presentation::handler;
use crate::shared::middleware::{activity_log_middleware, require_auth};

/// `DefaultBodyLimit` is scoped to just the `POST /files` route (via
/// `.layer()` on that one `MethodRouter` before it's merged into the path)
/// so every other route here keeps axum's normal small-body default --
/// only the upload endpoint needs to accept something up to
/// `config.storage.max_upload_bytes`.
pub fn routes(state: AppState) -> Router<AppState> {
    let max_upload_bytes = state.config.storage.max_upload_bytes;

    Router::new()
        .route("/files", get(handler::list_files))
        .route(
            "/files",
            post(handler::upload_file).layer(DefaultBodyLimit::max(max_upload_bytes)),
        )
        .route("/files/:uuid", get(handler::get_file))
        .route("/files/:uuid", delete(handler::delete_file))
        .route("/files/:uuid/download", get(handler::download_file))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            activity_log_middleware,
        ))
        .route_layer(axum::middleware::from_fn_with_state(state, require_auth))
}
