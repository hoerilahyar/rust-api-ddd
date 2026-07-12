use axum::Router;
use axum::routing::post;

use crate::bootstrap::state::AppState;
use crate::modules::auth::presentation::handler;

/// Public `/auth/*` routes -- no auth middleware, these are how you *get*
/// a token in the first place.
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/auth/login", post(handler::login))
        .route("/auth/refresh", post(handler::refresh))
        .route("/auth/logout", post(handler::logout))
        .route("/auth/forgot-password", post(handler::forgot_password))
        .route("/auth/reset-password", post(handler::reset_password))
}
