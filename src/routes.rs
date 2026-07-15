use axum::Router;

use crate::bootstrap::state::AppState;
use crate::modules::activity_log::presentation::routes as activity_log_routes;
use crate::modules::audit_auth_log::presentation::routes as audit_auth_routes;
use crate::modules::audit_trail_log::presentation::routes as audit_trail_routes;
use crate::modules::auth::presentation::routes as auth_routes;
use crate::modules::file::presentation::routes as file_routes;
use crate::modules::menu::presentation::routes as menu_routes;
use crate::modules::permission::presentation::routes as permission_routes;
use crate::modules::role::presentation::routes as role_routes;
use crate::modules::setting::presentation::routes as setting_routes;
use crate::modules::user::presentation::routes as user_routes;
use crate::modules::user_setting::presentation::routes as user_setting_routes;

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .merge(auth_routes::routes())
        .merge(user_routes::routes(state.clone()))
        .merge(role_routes::routes(state.clone()))
        .merge(permission_routes::routes(state.clone()))
        .merge(audit_auth_routes::routes(state.clone()))
        .merge(audit_trail_routes::routes(state.clone()))
        .merge(activity_log_routes::routes(state.clone()))
        .merge(file_routes::routes(state.clone()))
        .merge(menu_routes::routes(state.clone()))
        .merge(setting_routes::routes(state.clone()))
        .merge(user_setting_routes::routes(state.clone()))
}
