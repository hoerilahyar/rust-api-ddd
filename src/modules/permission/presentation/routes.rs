use axum::routing::{delete, get, post, put};
use axum::Router;

use crate::bootstrap::state::AppState;
use crate::modules::permission::presentation::handler;
use crate::shared::middleware::require_auth;

/// All `/permissions` + `/me` routes. Mounted under auth in `bootstrap::router`,
/// so every route here already requires a valid access token.
pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/permissions", get(handler::list_permissions))
        .route("/permissions", post(handler::create_permission))
        .route("/permissions/:id", get(handler::get_permission))
        .route("/permissions/:id", put(handler::update_permission))
        .route("/permissions/:id", delete(handler::delete_permission))
        .route_layer(axum::middleware::from_fn_with_state(state, require_auth))
}
