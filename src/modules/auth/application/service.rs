use async_trait::async_trait;

use crate::modules::auth::application::dto::TokenPairResponse;
use crate::shared::contracts::LoginAttempt;
use crate::shared::errors::AppError;

#[async_trait]
pub trait AuthService: Send + Sync {
    async fn login(
        &self,
        identifier: &str,
        password: &str,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<TokenPairResponse, AppError>;

    async fn refresh(&self, refresh_token: &str) -> Result<TokenPairResponse, AppError>;

    async fn logout(&self, refresh_token: &str) -> Result<(), AppError>;

    async fn forgot_password(&self, email: &str) -> Result<(), AppError>;

    async fn reset_password(&self, raw_token: &str, new_password: &str) -> Result<(), AppError>;

    /// Exposed mainly so handlers/tests can build a `LoginAttempt` without
    /// duplicating the audit-log wiring -- kept here rather than in the
    /// handler layer since "what counts as a login attempt" is a policy
    /// decision, not a transport concern.
    async fn record_attempt(&self, attempt: LoginAttempt) -> Result<(), AppError>;
}
