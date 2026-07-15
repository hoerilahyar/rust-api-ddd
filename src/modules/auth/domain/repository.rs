use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::modules::auth::domain::entity::{PasswordResetToken, RefreshToken};
use crate::shared::errors::AppError;

/// Persistence contract for everything auth owns: refresh tokens and
/// password reset tokens. Login-attempt auditing lives in the separate,
/// cross-module `shared::contracts::AuditAuthRecorder` trait.
#[async_trait]
pub trait AuthRepository: Send + Sync {
    async fn store_refresh_token(
        &self,
        user_id: i32,
        token_hash: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<RefreshToken, AppError>;

    async fn find_refresh_token_by_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<RefreshToken>, AppError>;

    async fn revoke_refresh_token(&self, id: Uuid) -> Result<(), AppError>;

    async fn revoke_all_refresh_tokens_for_user(&self, user_id: i32) -> Result<(), AppError>;

    async fn store_password_reset_token(
        &self,
        user_id: i32,
        token_hash: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<PasswordResetToken, AppError>;

    async fn find_password_reset_token_by_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<PasswordResetToken>, AppError>;

    async fn mark_password_reset_token_used(&self, id: Uuid) -> Result<(), AppError>;
}
