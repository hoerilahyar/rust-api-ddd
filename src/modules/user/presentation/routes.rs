use axum::Router;
use axum::routing::{delete, get, post, put};

use crate::bootstrap::state::AppState;
use crate::modules::user::presentation::handler;
use crate::shared::middleware::{activity_log_middleware, require_auth};

/// All `/users` + `/me` routes. Mounted under auth in `bootstrap::router`,
/// so every route here already requires a valid access token.
pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/me", get(handler::me))
        .route("/me/password", put(handler::change_my_password))
        .route("/users", get(handler::list_users))
        .route("/users", post(handler::create_user))
        .route("/users/:id", get(handler::get_user))
        .route("/users/:id", put(handler::update_user))
        .route("/users/:id", delete(handler::delete_user))
        .route("/users/:id/roles", post(handler::assign_role))
        .route("/users/:id/roles/:role_id", delete(handler::revoke_role))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            activity_log_middleware,
        ))
        .route_layer(axum::middleware::from_fn_with_state(state, require_auth))
}
