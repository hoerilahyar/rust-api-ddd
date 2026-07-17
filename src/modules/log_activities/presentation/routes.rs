use axum::routing::get;
use axum::Router;

use crate::bootstrap::state::AppState;
use crate::modules::log_activities::presentation::handler;
use crate::shared::middleware::{activity_log_middleware, require_auth};

/// All `/activity-logs` routes. Mounted under auth in `crate::routes`, so
/// every route here already requires a valid access token; `ensure_permission`
/// inside each handler further restricts access to `activity_log.read`.
pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/activity-logs", get(handler::list_activity_logs))
        .route("/activity-logs/:id", get(handler::get_activity_log))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            activity_log_middleware,
        ))
        .route_layer(axum::middleware::from_fn_with_state(state, require_auth))
}
