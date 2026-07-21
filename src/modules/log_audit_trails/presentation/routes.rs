use axum::routing::get;
use axum::Router;

use crate::bootstrap::state::AppState;
use crate::modules::log_audit_trails::presentation::handler;
use crate::shared::middleware::{activity_log_middleware, require_auth};

/// All `/audit-trail` routes. Mounted under auth in `bootstrap::router`, so every
/// route here already requires a valid access token; `ensure_permission`
/// inside each handler further restricts access to `audit.read`.
pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/audit-trail", get(handler::list_audit_trail_logs))
        .route("/audit-trail/:id", get(handler::get_audit_trail_log))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            activity_log_middleware,
        ))
        .route_layer(axum::middleware::from_fn_with_state(state, require_auth))
}
