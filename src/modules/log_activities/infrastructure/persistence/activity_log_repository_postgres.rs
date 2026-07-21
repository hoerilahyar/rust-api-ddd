use async_trait::async_trait;
use sqlx::{PgPool, Row};

use crate::modules::log_activities::domain::{
    ActivityLog, ActivityLogQuery, ActivityLogRepository,
};
use crate::shared::contracts::{Activity, ActivityRecorder, MethodRequest, Module, RecordActivity};
use crate::shared::errors::AppError;

/// SQLx/Postgres implementation of [`ActivityLogRepository`] (read side) and
/// [`ActivityRecorder`] (write side), targeting the schema defined by
/// `databases/postgresql/migrations/000015_create_log_activities.up.sql`.
#[derive(Clone)]
pub struct ActivityLogRepositoryPg {
    pool: PgPool,
}

impl ActivityLogRepositoryPg {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn map_row(row: &sqlx::postgres::PgRow) -> ActivityLog {
        ActivityLog {
            id: row.get("id"),
            user_id: row.get("user_id"),

            activity: Activity::from_str_or_view(row.get::<String, _>("activity").as_str()),
            module: Module::from_str_or_log(row.get::<String, _>("module").as_str()),

            resource_type: row.get("resource_type"),
            resource_id: row.get("resource_id"),

            method: MethodRequest::from_str_or_get(row.get::<String, _>("method").as_str()),
            path: row.get("path"),

            description: row.get("description"),

            ip_address: row.get("ip_address"),
            user_agent: row.get("user_agent"),

            status_code: row.get("status_code"),

            trace_id: row.get("trace_id"),

            created_at: row.get("created_at"),
        }
    }
}

#[async_trait]
impl ActivityLogRepository for ActivityLogRepositoryPg {
    async fn list(&self, query: &ActivityLogQuery) -> Result<(Vec<ActivityLog>, i64), AppError> {
        let (_page, limit) = query.normalized();
        let offset = query.offset();

        let search = query.search.clone().unwrap_or_default();
        let search_pattern = format!("%{search}%");

        // IMPORTANT: keep this as Option<String> (do NOT `.unwrap_or_default()`
        // into `""`) -- the SQL below relies on `$4` being a real SQL NULL when
        // no activity filter was given, so `$4::VARCHAR IS NULL` short-circuits
        // the clause. Defaulting to "" here would make every unfiltered list()
        // call silently match zero rows instead of returning everything.
        let activity = query.activity.map(|a| a.as_str().to_string());

        let rows = sqlx::query(
            r#"
            SELECT *
            FROM log_activities
            WHERE
                ($1 = ''
                    OR user_id::TEXT ILIKE $2
                    OR module ILIKE $2
                    OR path ILIKE $2
                    OR resource_type ILIKE $2
                    OR resource_id ILIKE $2
                    OR method ILIKE $2
                    OR description ILIKE $2)
                AND ($3::INT IS NULL OR user_id = $3)
                AND ($4::VARCHAR IS NULL OR activity = $4)
                AND ($5::TIMESTAMPTZ IS NULL OR created_at >= $5)
                AND ($6::TIMESTAMPTZ IS NULL OR created_at <= $6)
            ORDER BY created_at DESC
            LIMIT $7 OFFSET $8
            "#,
        )
        .bind(&search)
        .bind(&search_pattern)
        .bind(query.user_id)
        .bind(&activity)
        .bind(query.from)
        .bind(query.to)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let total: i64 = sqlx::query(
            r#"
            SELECT COUNT(*) AS total
            FROM log_activities
            WHERE
                ($1 = ''
                    OR user_id::TEXT ILIKE $2
                    OR module ILIKE $2
                    OR path ILIKE $2
                    OR resource_type ILIKE $2
                    OR resource_id ILIKE $2
                    OR method ILIKE $2
                    OR description ILIKE $2)
                AND ($3::INT IS NULL OR user_id = $3)
                AND ($4::VARCHAR IS NULL OR activity = $4)
                AND ($5::TIMESTAMPTZ IS NULL OR created_at >= $5)
                AND ($6::TIMESTAMPTZ IS NULL OR created_at <= $6)
            "#,
        )
        .bind(&search)
        .bind(&search_pattern)
        .bind(query.user_id)
        .bind(&activity)
        .bind(query.from)
        .bind(query.to)
        .fetch_one(&self.pool)
        .await?
        .get("total");

        let logs = rows.iter().map(Self::map_row).collect();

        Ok((logs, total))
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<ActivityLog>, AppError> {
        let row = sqlx::query(
            r#"
            SELECT *
            FROM log_activities
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| Self::map_row(&r)))
    }
}

#[async_trait]
impl ActivityRecorder for ActivityLogRepositoryPg {
    async fn record_activity(&self, activity: RecordActivity) -> Result<(), AppError> {
        sqlx::query(
            r#"
        INSERT INTO log_activities (
            user_id,
            activity,
            module,
            resource_type,
            resource_id,
            method,
            path,
            description,
            ip_address,
            user_agent,
            status_code,
            trace_id
        )
        VALUES (
            $1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12
        )
        "#,
        )
        .bind(activity.user_id)
        .bind(activity.activity.as_str())
        .bind(activity.module.as_str())
        .bind(activity.resource_type)
        .bind(activity.resource_id)
        .bind(activity.method.as_str())
        .bind(activity.path)
        .bind(activity.description)
        .bind(activity.ip_address)
        .bind(activity.user_agent)
        .bind(activity.status_code)
        .bind(activity.trace_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
