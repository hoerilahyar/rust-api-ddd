use axum::routing::{get, post};
use axum::Router;

use crate::bootstrap::state::AppState;
use crate::modules::masters::presentation::{group_handler, item_handler};
use crate::shared::middleware::{activity_log_middleware, require_auth};

/// All `/masters` + `/master-items` routes. Mounted under auth in
/// `bootstrap::router`, so every route here already requires a valid
/// access token.
pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        // ---- master groups ----
        .route(
            "/masters",
            get(group_handler::list_master_groups).post(group_handler::create_master_group),
        )
        .route(
            "/masters/:id",
            get(group_handler::get_master_group)
                .put(group_handler::update_master_group)
                .delete(group_handler::delete_master_group),
        )
        // ---- master items ----
        // nested create: group_id comes from the path, never trusted from body
        .route(
            "/masters/:group_id/items",
            post(item_handler::create_master_item),
        )
        .route("/master-items", get(item_handler::list_master_items))
        .route(
            "/master-items/:id",
            get(item_handler::get_master_item)
                .put(item_handler::update_master_item)
                .delete(item_handler::delete_master_item),
        )
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            activity_log_middleware,
        ))
        .route_layer(axum::middleware::from_fn_with_state(state, require_auth))
}
