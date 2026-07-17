use async_trait::async_trait;
use sqlx::{PgPool, Row};

use crate::modules::log_audit_auths::domain::{AuditAuthLogRepository, LoginLog, LoginLogQuery};
use crate::shared::contracts::LoginStatus;
use crate::shared::contracts::{AuditAuthRecorder, LoginAttempt};
use crate::shared::errors::AppError;

/// SQLx/Postgres implementation of [`AuditAuthLogRepository`], targeting the
/// schema defined by the migrations under `databases/postgresql`.
#[derive(Clone)]
pub struct AuditAuthLogRepositoryPg {
    pool: PgPool,
}

impl AuditAuthLogRepositoryPg {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn map_row(row: &sqlx::postgres::PgRow) -> LoginLog {
        let status: String = row.get("status");
        LoginLog {
            id: row.get("id"),
            user_id: row.get("user_id"),
            email_attempted: row.get("email_attempted"),
            ip_address: row.get("ip_address"),
            user_agent: row.get("user_agent"),
            status: if status == LoginStatus::Success.as_str() {
                LoginStatus::Success
            } else {
                LoginStatus::Failed
            },
            created_at: row.get("created_at"),
        }
    }
}

#[async_trait]
impl AuditAuthLogRepository for AuditAuthLogRepositoryPg {
    async fn list(&self, query: &LoginLogQuery) -> Result<(Vec<LoginLog>, i64), AppError> {
        let (_page, limit) = query.normalized();
        let offset = query.offset();
        let search = query.search.clone().unwrap_or_default();
        let search_pattern = format!("%{search}%");
        let status = query.status.map(|s| s.as_str().to_string());

        let rows = sqlx::query(
            r#"
            SELECT * FROM log_audit_auths
            WHERE ($1 = '' OR email_attempted ILIKE $2 OR ip_address ILIKE $2)
              AND ($3::INT IS NULL OR user_id = $3)
              AND ($4::VARCHAR IS NULL OR status = $4)
              AND ($5::TIMESTAMPTZ IS NULL OR created_at >= $5)
              AND ($6::TIMESTAMPTZ IS NULL OR created_at <= $6)
            ORDER BY created_at DESC
            LIMIT $7 OFFSET $8
            "#,
        )
        .bind(&search)
        .bind(&search_pattern)
        .bind(query.user_id)
        .bind(&status)
        .bind(query.from)
        .bind(query.to)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let total: i64 = sqlx::query(
            r#"
            SELECT COUNT(*) AS total FROM log_audit_auths
            WHERE ($1 = '' OR email_attempted ILIKE $2 OR ip_address ILIKE $2)
              AND ($3::INT IS NULL OR user_id = $3)
              AND ($4::VARCHAR IS NULL OR status = $4)
              AND ($5::TIMESTAMPTZ IS NULL OR created_at >= $5)
              AND ($6::TIMESTAMPTZ IS NULL OR created_at <= $6)
            "#,
        )
        .bind(&search)
        .bind(&search_pattern)
        .bind(query.user_id)
        .bind(&status)
        .bind(query.from)
        .bind(query.to)
        .fetch_one(&self.pool)
        .await?
        .get("total");

        let logs = rows.iter().map(Self::map_row).collect();
        Ok((logs, total))
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<LoginLog>, AppError> {
        let row = sqlx::query("SELECT * FROM log_audit_auths WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| Self::map_row(&r)))
    }
}

#[async_trait]
impl AuditAuthRecorder for AuditAuthLogRepositoryPg {
    async fn record_login_attempt(&self, attempt: LoginAttempt) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO log_audit_auths (user_id, email_attempted, ip_address, user_agent, status)
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
