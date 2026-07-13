use async_trait::async_trait;
use sqlx::{PgPool, Row};

use crate::modules::setting::domain::{SettingRepository, SystemSetting};
use crate::shared::errors::AppError;

/// SQLx/Postgres implementation of [`SettingRepository`], targeting the
/// schema defined by the migrations under `databases/postgresql`.
#[derive(Clone)]
pub struct SettingRepositoryPg {
    pool: PgPool,
}

impl SettingRepositoryPg {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn map_row(row: &sqlx::postgres::PgRow) -> SystemSetting {
        SystemSetting {
            id: row.get("id"),
            key: row.get("key"),
            value: row.get("value"),
            description: row.get("description"),
            updated_by: row.get("updated_by"),
            updated_at: row.get("updated_at"),
        }
    }
}

#[async_trait]
impl SettingRepository for SettingRepositoryPg {
    async fn list_all(&self) -> Result<Vec<SystemSetting>, AppError> {
        let rows = sqlx::query("SELECT * FROM system_settings ORDER BY key")
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.iter().map(Self::map_row).collect())
    }

    async fn find_by_key(&self, key: &str) -> Result<Option<SystemSetting>, AppError> {
        let row = sqlx::query("SELECT * FROM system_settings WHERE key = $1")
            .bind(key)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.as_ref().map(Self::map_row))
    }

    async fn upsert(
        &self,
        key: &str,
        value: Option<&str>,
        description: Option<&str>,
        updated_by: Option<i32>,
    ) -> Result<SystemSetting, AppError> {
        let row = sqlx::query(
            r#"
            INSERT INTO system_settings (key, value, description, updated_by, updated_at)
            VALUES ($1, $2, $3, $4, now())
            ON CONFLICT (key) DO UPDATE
                SET value       = EXCLUDED.value,
                    description = COALESCE(EXCLUDED.description, system_settings.description),
                    updated_by  = EXCLUDED.updated_by,
                    updated_at  = now()
            RETURNING *
            "#,
        )
        .bind(key)
        .bind(value)
        .bind(description)
        .bind(updated_by)
        .fetch_one(&self.pool)
        .await?;

        Ok(Self::map_row(&row))
    }

    async fn delete(&self, key: &str) -> Result<(), AppError> {
        sqlx::query("DELETE FROM system_settings WHERE key = $1")
            .bind(key)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
