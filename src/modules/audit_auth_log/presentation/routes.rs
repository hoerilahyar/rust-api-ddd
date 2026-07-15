use axum::routing::get;
use axum::Router;

use crate::bootstrap::state::AppState;
use crate::modules::audit_auth_log::presentation::handler;
use crate::shared::middleware::{activity_log_middleware, require_auth};

/// All `/audit-auth` routes. Mounted under auth in `bootstrap::router`, so every
/// route here already requires a valid access token; `ensure_permission`
/// inside each handler further restricts access to `audit.read`.
pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/audit-auth/logs", get(handler::list_audit_auth_logs))
        .route("/audit-auth/logs/:id", get(handler::get_audit_auth_log))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            activity_log_middleware,
        ))
        .route_layer(axum::middleware::from_fn_with_state(state, require_auth))
}
