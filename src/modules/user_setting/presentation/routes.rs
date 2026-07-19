use axum::routing::{delete, get, put};
use axum::Router;

use crate::bootstrap::state::AppState;
use crate::modules::user_setting::presentation::handler;
use crate::shared::middleware::{activity_log_middleware, require_auth};

/// `/me/settings` -- any authenticated user, always scoped to their own
/// `claims.sub` (see the handler doc comments). No extra permission check
/// needed beyond `require_auth`: a user managing their own preferences
/// doesn't need admin rights.
pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/me/settings", get(handler::list_my_settings))
        .route("/me/settings/:id", get(handler::get_my_setting))
        .route("/me/settings/:id", put(handler::upsert_my_setting))
        .route("/me/settings/:id", delete(handler::delete_my_setting))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            activity_log_middleware,
        ))
        .route_layer(axum::middleware::from_fn_with_state(state, require_auth))
}
