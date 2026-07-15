use std::sync::Arc;

use argon2::password_hash::{
    rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
};
use argon2::Argon2;
use async_trait::async_trait;
use chrono::{Duration as ChronoDuration, Utc};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::modules::auth::application::dto::{AuthUserSummary, TokenPairResponse};
use crate::modules::auth::application::service::AuthService;
use crate::modules::auth::domain::repository::AuthRepository;
use crate::modules::auth::infrastructure::jwt_service::JwtService;
use crate::shared::contracts::{
    AuditAuthRecorder, LoginAttempt, LoginStatus, UserAuthProjection, UserReader,
};
use crate::shared::errors::AppError;

pub struct AuthServiceImpl {
    auth_repo: Arc<dyn AuthRepository>,
    user_reader: Arc<dyn UserReader>,
    audit: Arc<dyn AuditAuthRecorder>,
    jwt: Arc<JwtService>,
}

fn hash_token(raw: &str) -> String {
    let digest = Sha256::digest(raw.as_bytes());
    hex::encode(digest)
}

fn to_summary(user: &UserAuthProjection) -> AuthUserSummary {
    AuthUserSummary {
        id: user.id,
        name: user.name.clone(),
        username: user.username.clone(),
        email: user.email.clone(),
        roles: user.roles.clone(),
        permissions: user.permissions.clone(),
    }
}

impl AuthServiceImpl {
    pub fn new(
        auth_repo: Arc<dyn AuthRepository>,
        user_reader: Arc<dyn UserReader>,
        audit: Arc<dyn AuditAuthRecorder>,
        jwt: Arc<JwtService>,
    ) -> Self {
        Self {
            auth_repo,
            user_reader,
            audit,
            jwt,
        }
    }

    async fn issue_tokens(&self, user: &UserAuthProjection) -> Result<TokenPairResponse, AppError> {
        let (access_token, expires_in) = self
            .jwt
            .generate_access_token(user)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("failed to sign access token: {e}")))?;

        let (refresh_token, refresh_ttl) = self.jwt.generate_refresh_token(user).map_err(|e| {
            AppError::Internal(anyhow::anyhow!("failed to sign refresh token: {e}"))
        })?;

        let expires_at = Utc::now() + ChronoDuration::seconds(refresh_ttl);
        self.auth_repo
            .store_refresh_token(user.id, &hash_token(&refresh_token), expires_at)
            .await?;

        Ok(TokenPairResponse {
            access_token,
            refresh_token,
            token_type: "Bearer",
            expires_in,
            user: to_summary(user),
        })
    }

    fn hash_password(&self, plain: &str) -> Result<String, AppError> {
        let salt = SaltString::generate(&mut OsRng);
        Argon2::default()
            .hash_password(plain.as_bytes(), &salt)
            .map(|h| h.to_string())
            .map_err(|e| AppError::Internal(anyhow::anyhow!("failed to hash password: {e}")))
    }

    fn verify_password(&self, plain: &str, hash: &str) -> Result<bool, AppError> {
        let parsed = PasswordHash::new(hash)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("invalid password hash: {e}")))?;
        Ok(Argon2::default()
            .verify_password(plain.as_bytes(), &parsed)
            .is_ok())
    }
}

#[async_trait]
impl AuthService for AuthServiceImpl {
    async fn login(
        &self,
        identifier: &str,
        password: &str,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Result<TokenPairResponse, AppError> {
        let user = self.user_reader.find_for_auth(identifier).await?;

        let user = match user {
            Some(u) => u,
            None => {
                self.audit
                    .record_login_attempt(LoginAttempt {
                        user_id: None,
                        email_attempted: Some(identifier.to_string()),
                        ip_address,
                        user_agent,
                        status: LoginStatus::Failed,
                    })
                    .await?;
                return Err(AppError::Unauthorized("invalid credentials".to_string()));
            }
        };

        if !user.is_active {
            self.audit
                .record_login_attempt(LoginAttempt {
                    user_id: Some(user.id),
                    email_attempted: Some(identifier.to_string()),
                    ip_address,
                    user_agent,
                    status: LoginStatus::Failed,
                })
                .await?;
            return Err(AppError::Forbidden("account is inactive".to_string()));
        }

        let password_ok = self.verify_password(password, &user.password_hash)?;
        if !password_ok {
            self.audit
                .record_login_attempt(LoginAttempt {
                    user_id: Some(user.id),
                    email_attempted: Some(identifier.to_string()),
                    ip_address,
                    user_agent,
                    status: LoginStatus::Failed,
                })
                .await?;
            return Err(AppError::Unauthorized("invalid credentials".to_string()));
        }

        self.audit
            .record_login_attempt(LoginAttempt {
                user_id: Some(user.id),
                email_attempted: Some(identifier.to_string()),
                ip_address,
                user_agent,
                status: LoginStatus::Success,
            })
            .await?;
        self.user_reader.touch_last_login(user.id).await?;

        self.issue_tokens(&user).await
    }

    async fn refresh(&self, refresh_token: &str) -> Result<TokenPairResponse, AppError> {
        // Verify signature/expiry first (cheap), then confirm it hasn't
        // been revoked server-side (DB lookup by hash).
        let claims = self
            .jwt
            .decode_refresh_token(refresh_token)
            .map_err(|_| AppError::Unauthorized("invalid or expired refresh token".to_string()))?;

        let stored = self
            .auth_repo
            .find_refresh_token_by_hash(&hash_token(refresh_token))
            .await?
            .ok_or_else(|| AppError::Unauthorized("refresh token not recognized".to_string()))?;

        if !stored.is_valid() {
            return Err(AppError::Unauthorized(
                "refresh token has been revoked or expired".to_string(),
            ));
        }

        let user = self
            .user_reader
            .find_by_id(claims.sub)
            .await?
            .ok_or_else(|| AppError::Unauthorized("user no longer exists".to_string()))?;

        if !user.is_active {
            return Err(AppError::Forbidden("account is inactive".to_string()));
        }

        // Rotate: revoke the used refresh token and issue a brand new pair.
        self.auth_repo.revoke_refresh_token(stored.id).await?;
        self.issue_tokens(&user).await
    }

    async fn logout(&self, refresh_token: &str) -> Result<(), AppError> {
        if let Some(stored) = self
            .auth_repo
            .find_refresh_token_by_hash(&hash_token(refresh_token))
            .await?
        {
            self.auth_repo.revoke_refresh_token(stored.id).await?;
        }
        Ok(())
    }

    async fn forgot_password(&self, email: &str) -> Result<(), AppError> {
        // Always succeed from the caller's perspective (don't leak whether
        // an email is registered); only actually create a token if it is.
        if let Some(user) = self.user_reader.find_for_auth(email).await? {
            let raw_token = Uuid::new_v4().to_string();
            let expires_at = Utc::now() + ChronoDuration::hours(1);
            // println!("token: {}", raw_token);
            self.auth_repo
                .store_password_reset_token(user.id, &hash_token(&raw_token), expires_at)
                .await?;

            // In a real deployment this token would be emailed to the user,
            // never logged. Left as a TODO integration point (email/queue).
            tracing::info!(user_id = user.id, "password reset token generated");
        }
        Ok(())
    }

    async fn reset_password(&self, raw_token: &str, new_password: &str) -> Result<(), AppError> {
        let stored = self
            .auth_repo
            .find_password_reset_token_by_hash(&hash_token(raw_token))
            .await?
            .ok_or_else(|| AppError::Unauthorized("invalid or expired reset token".to_string()))?;

        if !stored.is_valid() {
            return Err(AppError::Unauthorized(
                "invalid or expired reset token".to_string(),
            ));
        }

        let new_hash = self.hash_password(new_password)?;
        self.user_reader
            .update_password(stored.user_id, &new_hash)
            .await?;
        self.auth_repo
            .mark_password_reset_token_used(stored.id)
            .await?;
        self.auth_repo
            .revoke_all_refresh_tokens_for_user(stored.user_id)
            .await?;
        Ok(())
    }

    async fn record_attempt(&self, attempt: LoginAttempt) -> Result<(), AppError> {
        self.audit.record_login_attempt(attempt).await
    }
}
