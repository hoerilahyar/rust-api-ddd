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
        // nested create: group id comes from the path, never trusted from body.
        // NOTE: this segment must reuse the same param name (`:id`) as
        // `/masters/:id` above -- axum's router (matchit) requires every
        // route sharing a dynamic segment position to use the same
        // parameter name, even across otherwise-unrelated routes. Using a
        // different name here (e.g. `:group_id`) makes the whole router
        // panic at startup with "insertion failed due to conflict with
        // previously registered route". The extractor in the handler below
        // still binds it to `group_id` (Path<i64> doesn't care about the
        // literal name in the route string), so nothing else changes.
        .route(
            "/masters/:id/items",
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
