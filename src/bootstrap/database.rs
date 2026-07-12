use sqlx::postgres::{PgPool, PgPoolOptions};

use crate::bootstrap::config::DatabaseConfig;

/// Creates the shared Postgres connection pool used by every repository.
pub async fn connect(config: &DatabaseConfig) -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .connect(&config.url)
        .await?;

    tracing::info!("connected to postgres");
    Ok(pool)
}
