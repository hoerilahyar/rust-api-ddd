use axum::{extract::State, routing::get, Json, Router};
use serde::Serialize;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::bootstrap::state::AppState;
use crate::routes;
use crate::shared::middleware::rate_limit_middleware;

#[derive(Serialize)]
pub struct HealthResponse {
    status: &'static str,
    service: &'static str,
    version: &'static str,
    branch: &'static str,
    commit: &'static str,
    build_time: &'static str,
    run_time: String,
}

/// Builds the full application router.
pub fn build_router(state: AppState) -> Router {
    let api = routes::routes(state.clone());

    Router::new()
        .nest("/api/v1", api)
        .route("/health", get(health_check))
        .layer(
            ServiceBuilder::new()
                .layer(crate::shared::middleware::recover_layer())
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive())
                .layer(axum::middleware::from_fn_with_state(
                    state.clone(),
                    rate_limit_middleware,
                )),
        )
        .with_state(state)
}

async fn health_check(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "UP",
        service: env!("CARGO_PKG_NAME"),
        version: env!("CARGO_PKG_VERSION"),
        branch: env!("GIT_BRANCH"),
        commit: env!("GIT_COMMIT"),
        build_time: env!("BUILD_TIME"),
        run_time: state.started_at.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
    })
}
