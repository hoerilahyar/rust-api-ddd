use axum::extract::{ConnectInfo, State};
use axum::http::HeaderMap;
use axum::response::IntoResponse;
use std::net::SocketAddr;
use uuid::Uuid;

use crate::bootstrap::state::AppState;
use crate::modules::auth::application::dto::{
    ForgotPasswordRequest, LoginRequest, LogoutRequest, RefreshRequest, ResetPasswordRequest,
};
use crate::shared::contracts::{Activity, MethodRequest, Module, RecordActivity};
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

/// Fire-and-forget activity trail entry for one of the `/auth/*` actions.
///
/// `/auth/*` routes are public (no `require_auth` layer), so they never pass
/// through `activity_log_middleware` -- that middleware only fires *inside*
/// `require_auth`, once `Claims` are already in the request's extensions.
/// `login` itself is already covered by `AuditRecorder` / `user_login_logs`
/// (see `auth_service.login`); this covers the remaining actions
/// (`refresh`, `logout`, `forgot_password`, `reset_password`) so every
/// security-sensitive auth action ends up in `activity_logs` too, same as
/// every other module.
fn record_auth_activity(
    state: &AppState,
    user_id: Option<i32>,
    activity: Activity,
    method: MethodRequest,
    path: &'static str,
    description: Option<String>,
    ip_address: Option<String>,
    user_agent: Option<String>,
) {
    let recorder = state.activity_recorder.clone();
    let record = RecordActivity {
        user_id,
        activity,
        module: Module::Auth,
        resource_type: Some("auth".to_string()),
        resource_id: None,
        method,
        path: path.to_string(),
        description,
        ip_address,
        user_agent,
        status_code: None,
        trace_id: Some(Uuid::new_v4()),
    };

    tokio::spawn(async move {
        if let Err(err) = recorder.record_activity(record).await {
            tracing::error!(error = ?err, "failed to record auth activity log");
        }
    });
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
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    ValidatedJson(payload): ValidatedJson<RefreshRequest>,
) -> Result<impl IntoResponse, AppError> {
    let ip = client_ip(&headers, addr);
    let ua = user_agent(&headers);

    let tokens = state.auth_service.refresh(&payload.refresh_token).await?;

    record_auth_activity(
        &state,
        Some(tokens.user.id),
        Activity::Refresh,
        MethodRequest::Post,
        "/auth/refresh",
        None,
        ip,
        ua,
    );

    Ok(ApiResponse::new("token refreshed", tokens))
}

pub async fn logout(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    ValidatedJson(payload): ValidatedJson<LogoutRequest>,
) -> Result<impl IntoResponse, AppError> {
    let ip = client_ip(&headers, addr);
    let ua = user_agent(&headers);

    // Best-effort: identify who's logging out from the refresh token they
    // presented, before it gets invalidated. A token that fails to decode
    // (expired/malformed) still logs out fine -- we just record it without
    // a user_id, same as an unmatched email in `LoginAttempt`.
    let user_id = state
        .jwt
        .decode_refresh_token(&payload.refresh_token)
        .ok()
        .map(|claims| claims.sub);

    state.auth_service.logout(&payload.refresh_token).await?;

    record_auth_activity(
        &state,
        user_id,
        Activity::Logout,
        MethodRequest::Post,
        "/auth/logout",
        None,
        ip,
        ua,
    );

    Ok(ApiResponse::message("logged out"))
}

pub async fn forgot_password(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    ValidatedJson(payload): ValidatedJson<ForgotPasswordRequest>,
) -> Result<impl IntoResponse, AppError> {
    let ip = client_ip(&headers, addr);
    let ua = user_agent(&headers);

    state.auth_service.forgot_password(&payload.email).await?;

    // No user_id here by design -- this endpoint deliberately doesn't reveal
    // whether the email exists (see the response message below), so the
    // activity log shouldn't leak that either. The email itself goes in
    // `description` for support/investigation purposes.
    record_auth_activity(
        &state,
        None,
        Activity::ForgotPassword,
        MethodRequest::Post,
        "/auth/forgot-password",
        Some(format!("requested for {}", payload.email)),
        ip,
        ua,
    );

    Ok(ApiResponse::message(
        "if that email is registered, a password reset link has been sent",
    ))
}

pub async fn reset_password(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    ValidatedJson(payload): ValidatedJson<ResetPasswordRequest>,
) -> Result<impl IntoResponse, AppError> {
    let ip = client_ip(&headers, addr);
    let ua = user_agent(&headers);

    state
        .auth_service
        .reset_password(&payload.token, &payload.password)
        .await?;

    record_auth_activity(
        &state,
        None,
        Activity::ResetPassword,
        MethodRequest::Post,
        "/auth/reset-password",
        None,
        ip,
        ua,
    );

    Ok(ApiResponse::message("password has been reset"))
}
