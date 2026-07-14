use axum::{
    extract::{ConnectInfo, Request, State},
    http::HeaderMap,
    middleware::Next,
    response::Response,
};
use redis::AsyncCommands;
use std::net::{IpAddr, SocketAddr};

use crate::bootstrap::state::AppState;
use crate::shared::errors::AppError;

/// Resolves the client IP used as the rate-limit key. Only reads the
/// client-supplied `X-Forwarded-For` header when `trust_proxy` is true --
/// i.e. when we know there's a trusted reverse proxy in front of us that
/// sets/overwrites it. Otherwise a client could send an arbitrary/rotating
/// value in that header and bypass rate limiting entirely, since anyone can
/// set it. When multiple IPs are chained (`client, proxy1, proxy2`), the
/// first one is the original client.
pub fn resolve_client_ip(headers: &HeaderMap, socket_ip: IpAddr, trust_proxy: bool) -> String {
    if trust_proxy {
        headers
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.split(',').next())
            .map(|v| v.trim().to_string())
            .unwrap_or_else(|| socket_ip.to_string())
    } else {
        socket_ip.to_string()
    }
}

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

    let ip = resolve_client_ip(req.headers(), addr.ip(), state.config.rate_limit.trust_proxy);

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

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    fn socket_ip() -> IpAddr {
        "203.0.113.9".parse().unwrap()
    }

    #[test]
    fn ignores_forwarded_header_when_proxy_not_trusted() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", HeaderValue::from_static("198.51.100.1"));

        // A client can set this header to anything -- without trust_proxy,
        // it must be ignored entirely, or rate limiting is trivially bypassed.
        let ip = resolve_client_ip(&headers, socket_ip(), false);
        assert_eq!(ip, "203.0.113.9");
    }

    #[test]
    fn uses_forwarded_header_when_proxy_trusted() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", HeaderValue::from_static("198.51.100.1"));

        let ip = resolve_client_ip(&headers, socket_ip(), true);
        assert_eq!(ip, "198.51.100.1");
    }

    #[test]
    fn takes_first_ip_in_forwarded_chain() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-forwarded-for",
            HeaderValue::from_static("198.51.100.1, 10.0.0.1, 10.0.0.2"),
        );

        let ip = resolve_client_ip(&headers, socket_ip(), true);
        assert_eq!(ip, "198.51.100.1");
    }

    #[test]
    fn falls_back_to_socket_ip_when_header_missing_even_if_trusted() {
        let headers = HeaderMap::new();
        let ip = resolve_client_ip(&headers, socket_ip(), true);
        assert_eq!(ip, "203.0.113.9");
    }
}
