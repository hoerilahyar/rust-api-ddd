use axum::{
    extract::{ConnectInfo, Request, State},
    middleware::Next,
    response::Response,
};
use redis::AsyncCommands;
use std::net::SocketAddr;

use crate::bootstrap::state::AppState;
use crate::shared::errors::AppError;

/// Fixed-window rate limiter backed by Redis (`INCR` + `EXPIRE`), keyed by
/// client IP and path. Config comes from `AppState.config.rate_limit`.
pub async fn rate_limit_middleware(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let limit = state.config.rate_limit.max_requests;
    let window_secs = state.config.rate_limit.window_seconds;

    // Only read the client-supplied `X-Forwarded-For` header when we know
    // there's a trusted reverse proxy in front of us that sets/overwrites
    // it. Otherwise a client could send an arbitrary/rotating value and
    // bypass rate limiting entirely, since anyone can set this header.
    let ip = if state.config.rate_limit.trust_proxy {
        req.headers()
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.split(',').next())
            .map(|v| v.trim().to_string())
            .unwrap_or_else(|| addr.ip().to_string())
    } else {
        addr.ip().to_string()
    };

    let key = format!("rate_limit:{ip}:{}", req.uri().path());

    let mut conn = state.redis.clone();
    let count: i64 = conn
        .incr(&key, 1)
        .await
        .map_err(|e| AppError::Cache(e.to_string()))?;

    if count == 1 {
        let _: () = conn
            .expire(&key, window_secs as i64)
            .await
            .map_err(|e| AppError::Cache(e.to_string()))?;
    }

    if count > limit as i64 {
        return Err(AppError::TooManyRequests);
    }

    Ok(next.run(req).await)
}
