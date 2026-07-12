use redis::Client;
use redis::aio::ConnectionManager;

use crate::bootstrap::config::RedisConfig;

/// Creates a Redis `ConnectionManager`: a cheap-to-clone, auto-reconnecting
/// async connection shared across the whole app (cache, rate limiter,
/// refresh-token/session storage).
pub async fn connect(config: &RedisConfig) -> anyhow::Result<ConnectionManager> {
    let client = Client::open(config.url.clone())?;
    let manager = client.get_connection_manager().await?;

    tracing::info!("connected to redis");
    Ok(manager)
}
