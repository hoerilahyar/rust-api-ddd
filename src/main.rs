use std::net::SocketAddr;

use rust_clean_ddd::bootstrap::{config::AppConfig, database, migration, redis, router, state};
use rust_clean_ddd::shared::logger;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = AppConfig::load()?;
    logger::init(&config.server.log_level);

    let db = database::connect(&config.database).await?;

    if config.database.run_migrations_on_boot {
        migration::run_migrations(&db).await?;
        // migration::run_seeders(&db, "databases/postgresql/seeders").await?;
    }

    let redis_conn = redis::connect(&config.redis).await?;

    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port).parse()?;

    let state = state::AppState::new(config, db, redis_conn);
    let app = router::build_router(state);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(%addr, "server listening");

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}
