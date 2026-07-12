use axum::extract::{ConnectInfo, State};
use axum::http::HeaderMap;
use axum::response::IntoResponse;
use std::net::SocketAddr;

use crate::bootstrap::state::AppState;
use crate::modules::auth::application::dto::{
    ForgotPasswordRequest, LoginRequest, LogoutRequest, RefreshRequest, ResetPasswordRequest,
};
use crate::shared::errors::AppError;
use crate::shared::response::ApiResponse;
use crate::shared::validator::ValidatedJson;

fn client_ip(headers: &HeaderMap, addr: SocketAddr) -> Option<String> {
    headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .map(|v| v.trim().to_string())
        .or_else(|| Some(addr.ip().to_string()))
}

fn user_agent(headers: &HeaderMap) -> Option<String> {
    headers
        .get(axum::http::header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.to_string())
}

pub async fn login(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    ValidatedJson(payload): ValidatedJson<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    let ip = client_ip(&headers, addr);
    let ua = user_agent(&headers);

    let tokens = state
        .auth_service
        .login(&payload.identifier, &payload.password, ip, ua)
        .await?;

    Ok(ApiResponse::new("login successful", tokens))
}

pub async fn refresh(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<RefreshRequest>,
) -> Result<impl IntoResponse, AppError> {
    let tokens = state.auth_service.refresh(&payload.refresh_token).await?;
    Ok(ApiResponse::new("token refreshed", tokens))
}

pub async fn logout(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<LogoutRequest>,
) -> Result<impl IntoResponse, AppError> {
    state.auth_service.logout(&payload.refresh_token).await?;
    Ok(ApiResponse::message("logged out"))
}

pub async fn forgot_password(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<ForgotPasswordRequest>,
) -> Result<impl IntoResponse, AppError> {
    state.auth_service.forgot_password(&payload.email).await?;
    Ok(ApiResponse::message(
        "if that email is registered, a password reset link has been sent",
    ))
}

pub async fn reset_password(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<ResetPasswordRequest>,
) -> Result<impl IntoResponse, AppError> {
    state
        .auth_service
        .reset_password(&payload.token, &payload.password)
        .await?;
    Ok(ApiResponse::message("password has been reset"))
}
