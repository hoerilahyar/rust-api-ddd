use axum::routing::{delete, get, post, put};
use axum::Router;

use crate::bootstrap::state::AppState;
use crate::modules::menu::presentation::handler;
use crate::shared::middleware::{activity_log_middleware, require_auth};

/// `/me/menu` is available to any authenticated user (filtered to what
/// their permissions allow -- see `MenuService::visible_tree`). Every other
/// route here additionally requires `menu.manage`, checked per-handler via
/// `ensure_permission`. All of it sits behind `require_auth` regardless.
pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/me/menu", get(handler::get_my_menu))
        .route("/menus/tree", get(handler::get_menu_tree))
        .route("/menus", get(handler::list_menus))
        .route("/menus", post(handler::create_menu))
        .route("/menus/:id", get(handler::get_menu))
        .route("/menus/:id", put(handler::update_menu))
        .route("/menus/:id", delete(handler::delete_menu))
        .route("/menus/:id/permissions", put(handler::sync_permissions))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            activity_log_middleware,
        ))
        .route_layer(axum::middleware::from_fn_with_state(state, require_auth))
}
