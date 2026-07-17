use async_trait::async_trait;
use sqlx::{PgPool, Row};

use crate::modules::log_audit_trails::domain::{
    AuditTrailLog, AuditTrailLogQuery, AuditTrailLogRepository,
};
use crate::shared::contracts::AuditTrailRecorder;
use crate::shared::errors::AppError;

/// SQLx/Postgres implementation of [`AuditTrailLogRepository`], targeting the
/// schema defined by the migrations under `databases/postgresql`.
#[derive(Clone)]
pub struct AuditTrailLogRepositoryPg {
    pool: PgPool,
}

impl AuditTrailLogRepositoryPg {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn map_row(row: &sqlx::postgres::PgRow) -> AuditTrailLog {
        AuditTrailLog {
            id: row.get("id"),
            action: row.get("action"),
            entity_type: row.get("entity_type"),
            entity_id: row.get("entity_id"),
            old_values: row.get("old_values"),
            new_values: row.get("new_values"),
            ip_address: row.get("ip_address"),
            user_agent: row.get("user_agent"),
            description: row.get("description"),
            user_id: row.get("user_id"),
            created_at: row.get("created_at"),
        }
    }
}

#[async_trait]
impl AuditTrailLogRepository for AuditTrailLogRepositoryPg {
    async fn list(
        &self,
        query: &AuditTrailLogQuery,
    ) -> Result<(Vec<AuditTrailLog>, i64), AppError> {
        let (_page, limit) = query.normalized();
        let offset = query.offset();
        let search = query.search.clone().unwrap_or_default();
        let search_pattern = format!("%{search}%");

        let rows = sqlx::query(
            r#"
        SELECT * FROM log_audit_trails
        WHERE ($1 = '' OR entity_type ILIKE $2 OR entity_id::TEXT ILIKE $2 OR ip_address ILIKE $2)
          AND ($3::INT IS NULL OR user_id = $3)
          AND ($4::VARCHAR IS NULL OR action = $4)
          AND ($5::TIMESTAMPTZ IS NULL OR created_at >= $5)
          AND ($6::TIMESTAMPTZ IS NULL OR created_at <= $6)
        ORDER BY created_at DESC
        LIMIT $7 OFFSET $8
        "#,
        )
        .bind(&search)
        .bind(&search_pattern)
        .bind(query.user_id)
        .bind(query.action.clone())
        .bind(query.from)
        .bind(query.to)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let total: i64 = sqlx::query(
            r#"
        SELECT COUNT(*) AS total FROM log_audit_trails
        WHERE ($1 = '' OR entity_type ILIKE $2 OR entity_id::TEXT ILIKE $2 OR ip_address ILIKE $2)
          AND ($3::INT IS NULL OR user_id = $3)
          AND ($4::VARCHAR IS NULL OR action = $4)
          AND ($5::TIMESTAMPTZ IS NULL OR created_at >= $5)
          AND ($6::TIMESTAMPTZ IS NULL OR created_at <= $6)
        "#,
        )
        .bind(&search)
        .bind(&search_pattern)
        .bind(query.user_id)
        .bind(query.action.clone())
        .bind(query.from)
        .bind(query.to)
        .fetch_one(&self.pool)
        .await?
        .get("total");

        let logs = rows.iter().map(Self::map_row).collect();
        Ok((logs, total))
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<AuditTrailLog>, AppError> {
        let row = sqlx::query("SELECT * FROM log_audit_trails WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| Self::map_row(&r)))
    }
}

#[async_trait]
impl AuditTrailRecorder for AuditTrailLogRepositoryPg {
    async fn record_audit_trail_log(&self, log: AuditTrailLog) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO log_audit_trails 
                    (user_id, action, entity_type, entity_id, old_values, new_values, ip_address, user_agent, description)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(log.user_id)
        .bind(log.action)
        .bind(log.entity_type)
        .bind(log.entity_id)
        .bind(log.old_values)
        .bind(log.new_values)
        .bind(log.ip_address)
        .bind(log.user_agent)
        .bind(log.description)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
