use axum::Router;

use crate::bootstrap::state::AppState;
use crate::modules::audit::presentation::routes as audit_routes;
use crate::modules::auth::presentation::routes as auth_routes;
use crate::modules::menu::presentation::routes as menu_routes;
use crate::modules::permission::presentation::routes as permission_routes;
use crate::modules::role::presentation::routes as role_routes;
use crate::modules::user::presentation::routes as user_routes;

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .merge(auth_routes::routes())
        .merge(user_routes::routes(state.clone()))
        .merge(role_routes::routes(state.clone()))
        .merge(permission_routes::routes(state.clone()))
        .merge(audit_routes::routes(state.clone()))
        .merge(menu_routes::routes(state.clone()))
}
