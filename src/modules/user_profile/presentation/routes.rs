use axum::routing::{delete, get, put};
use axum::Router;

use crate::bootstrap::state::AppState;
use crate::modules::user_profile::presentation::handler;
use crate::shared::middleware::{activity_log_middleware, require_auth};

/// `/me/profile` -- any authenticated user, always scoped to their own
/// `claims.sub` (see the handler doc comments). `/users/:id/profile` is the
/// one admin-facing exception, gated per-handler behind `user.manage`.
pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/me/profile", get(handler::get_my_profile))
        .route("/me/profile", put(handler::upsert_my_profile))
        .route("/me/profile", delete(handler::delete_my_profile))
        .route("/users/:id/profile", get(handler::get_user_profile))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            activity_log_middleware,
        ))
        .route_layer(axum::middleware::from_fn_with_state(state, require_auth))
}
