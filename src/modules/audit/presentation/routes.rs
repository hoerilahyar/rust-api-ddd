use axum::routing::get;
use axum::Router;

use crate::bootstrap::state::AppState;
use crate::modules::audit::presentation::handler;
use crate::shared::middleware::require_auth;

/// All `/audit` routes. Mounted under auth in `bootstrap::router`, so every
/// route here already requires a valid access token; `ensure_permission`
/// inside each handler further restricts access to `audit.read`.
pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/audit/login-logs", get(handler::list_login_logs))
        .route("/audit/login-logs/:id", get(handler::get_login_log))
        .route_layer(axum::middleware::from_fn_with_state(state, require_auth))
}
