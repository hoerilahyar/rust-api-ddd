use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenType {
    Access,
    Refresh,
}

/// JWT payload. Carried through the request via `Extension<Claims>` after
/// `shared::middleware::require_auth` verifies the access token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject: the user id.
    pub sub: i32,
    pub username: String,
    pub email: String,
    #[serde(default)]
    pub roles: Vec<String>,
    #[serde(default)]
    pub permissions: Vec<String>,
    pub token_type: TokenType,
    /// Issued-at (unix seconds).
    pub iat: i64,
    /// Expiry (unix seconds) -- checked automatically by `jsonwebtoken`.
    pub exp: i64,
}
