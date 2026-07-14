use axum::routing::{delete, get, put};
use axum::Router;

use crate::bootstrap::state::AppState;
use crate::modules::setting::presentation::handler;
use crate::shared::middleware::{activity_log_middleware, require_auth};

/// All `/settings` routes require `settings.manage` (checked per-handler).
/// Mounted under auth in `bootstrap::router`, so a valid access token is
/// required regardless.
pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/settings", get(handler::list_settings))
        .route("/settings/:key", get(handler::get_setting))
        .route("/settings/:key", put(handler::upsert_setting))
        .route("/settings/:key", delete(handler::delete_setting))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            activity_log_middleware,
        ))
        .route_layer(axum::middleware::from_fn_with_state(state, require_auth))
}
