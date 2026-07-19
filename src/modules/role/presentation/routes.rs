use axum::routing::{delete, get, post, put};
use axum::Router;

use crate::bootstrap::state::AppState;
use crate::modules::role::presentation::handler;
use crate::shared::middleware::{activity_log_middleware, require_auth};

/// All `/roles` + `/me` routes. Mounted under auth in `bootstrap::router`,
/// so every route here already requires a valid access token.
pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/roles", get(handler::list_roles))
        .route("/roles", post(handler::create_role))
        .route("/roles/:id", get(handler::get_role))
        .route("/roles/:id", put(handler::update_role))
        .route("/roles/:id", delete(handler::delete_role))
        .route("/roles/:id/permission", post(handler::assign_permission))
        .route(
            "/roles/:id/permission/:permission_id",
            delete(handler::revoke_permission),
        )
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            activity_log_middleware,
        ))
        .route_layer(axum::middleware::from_fn_with_state(state, require_auth))
}
