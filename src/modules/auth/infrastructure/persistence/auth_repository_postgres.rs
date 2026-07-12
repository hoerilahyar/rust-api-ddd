use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::modules::auth::domain::entity::{PasswordResetToken, RefreshToken};
use crate::modules::auth::domain::repository::AuthRepository;
use crate::shared::contracts::{AuditRecorder, LoginAttempt};
use crate::shared::errors::AppError;

/// SQLx/Postgres implementation of [`AuthRepository`] + [`AuditRecorder`].
///
/// NOTE: kept as `auth_repository_postgres.rs` to match the existing module
/// skeleton (`pub mod auth_repository_postgres;`); the actual backend targeted
/// by the bundled migrations is PostgreSQL, not PostgreSQL.
#[derive(Clone)]
pub struct AuthRepositoryPg {
    pool: PgPool,
}

impl AuthRepositoryPg {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn map_refresh_token(row: &sqlx::postgres::PgRow) -> RefreshToken {
        RefreshToken {
            id: row.get("id"),
            user_id: row.get("user_id"),
            token_hash: row.get("token_hash"),
            expires_at: row.get("expires_at"),
            revoked_at: row.get("revoked_at"),
            created_at: row.get("created_at"),
        }
    }

    fn map_reset_token(row: &sqlx::postgres::PgRow) -> PasswordResetToken {
        PasswordResetToken {
            id: row.get("id"),
            user_id: row.get("user_id"),
            token_hash: row.get("token_hash"),
            expires_at: row.get("expires_at"),
            used_at: row.get("used_at"),
            created_at: row.get("created_at"),
        }
    }
}

#[async_trait]
impl AuthRepository for AuthRepositoryPg {
    async fn store_refresh_token(
        &self,
        user_id: i32,
        token_hash: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<RefreshToken, AppError> {
        let row = sqlx::query(
            r#"
            INSERT INTO refresh_tokens (user_id, token_hash, expires_at)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
        )
        .bind(user_id)
        .bind(token_hash)
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(Self::map_refresh_token(&row))
    }

    async fn find_refresh_token_by_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<RefreshToken>, AppError> {
        let row = sqlx::query("SELECT * FROM refresh_tokens WHERE token_hash = $1")
            .bind(token_hash)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| Self::map_refresh_token(&r)))
    }

    async fn revoke_refresh_token(&self, id: Uuid) -> Result<(), AppError> {
        sqlx::query(
            "UPDATE refresh_tokens SET revoked_at = now() WHERE id = $1 AND revoked_at IS NULL",
        )
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn revoke_all_refresh_tokens_for_user(&self, user_id: i32) -> Result<(), AppError> {
        sqlx::query("UPDATE refresh_tokens SET revoked_at = now() WHERE user_id = $1 AND revoked_at IS NULL")
            .bind(user_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn store_password_reset_token(
        &self,
        user_id: i32,
        token_hash: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<PasswordResetToken, AppError> {
        let row = sqlx::query(
            r#"
            INSERT INTO password_reset_tokens (user_id, token_hash, expires_at)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
        )
        .bind(user_id)
        .bind(token_hash)
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(Self::map_reset_token(&row))
    }

    async fn find_password_reset_token_by_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<PasswordResetToken>, AppError> {
        let row = sqlx::query("SELECT * FROM password_reset_tokens WHERE token_hash = $1")
            .bind(token_hash)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| Self::map_reset_token(&r)))
    }

    async fn mark_password_reset_token_used(&self, id: Uuid) -> Result<(), AppError> {
        sqlx::query(
            "UPDATE password_reset_tokens SET used_at = now() WHERE id = $1 AND used_at IS NULL",
        )
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

#[async_trait]
impl AuditRecorder for AuthRepositoryPg {
    async fn record_login_attempt(&self, attempt: LoginAttempt) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO user_login_logs (user_id, email_attempted, ip_address, user_agent, status)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(attempt.user_id)
        .bind(attempt.email_attempted)
        .bind(attempt.ip_address)
        .bind(attempt.user_agent)
        .bind(attempt.status.as_str())
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
