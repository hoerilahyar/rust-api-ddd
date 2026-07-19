use axum::{
    extract::{Request, State},
    http::header,
    middleware::Next,
    response::Response,
};

use crate::bootstrap::state::AppState;
use crate::modules::auth::domain::value_object::Claims;
use crate::shared::authz_cache::get_snapshot;
use crate::shared::errors::AppError;

/// Verifies the `Authorization: Bearer <jwt>` header, decodes the access
/// token, and injects [`Claims`] into request extensions so downstream
/// handlers/extractors can pull the authenticated user out via
/// `Extension<Claims>`.
///
/// The token's own `roles`/`permissions` are only used to identify *who* is
/// asking (via `sub`) -- they are a snapshot from whenever the token was
/// minted and may already be stale. Before the request is allowed through,
/// this middleware overwrites `roles`/`permissions` with a **live** lookup
/// (see `shared::authz_cache`) and rejects the request outright if the
/// account has since been deactivated. This is what makes a role, menu, or
/// permission change (or a deactivation) take effect on the user's very
/// next request, rather than only after their access token expires or they
/// log out and back in.
pub async fn require_auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let token = extract_bearer_token(&req)?;
    let mut claims = state
        .jwt
        .decode_access_token(&token)
        .map_err(|_| AppError::Unauthorized("invalid or expired token".to_string()))?;

    let snapshot = get_snapshot(state.cache.as_ref(), state.user_reader.as_ref(), claims.sub)
        .await?
        .ok_or_else(|| AppError::Unauthorized("user no longer exists".to_string()))?;

    if !snapshot.is_active {
        return Err(AppError::Forbidden("account is inactive".to_string()));
    }

    claims.roles = snapshot.roles;
    claims.permissions = snapshot.permissions;

    req.extensions_mut().insert(claims);
    Ok(next.run(req).await)
}

fn extract_bearer_token(req: &Request) -> Result<String, AppError> {
    let header_value = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized("missing authorization header".to_string()))?;

    header_value
        .strip_prefix("Bearer ")
        .map(|t| t.to_string())
        .ok_or_else(|| AppError::Unauthorized("authorization header must be a Bearer token".to_string()))
}

/// RBAC helper for handlers: `require_auth` already put [`Claims`] in
/// extensions, so a handler that needs a specific permission just does
/// `ensure_permission(&claims, "user.manage")?` at the top of its body.
pub fn ensure_permission(claims: &Claims, permission: &str) -> Result<(), AppError> {
    if claims.permissions.iter().any(|p| p == permission) {
        Ok(())
    } else {
        Err(AppError::Forbidden(format!(
            "missing required permission: {permission}"
        )))
    }
}
