use async_trait::async_trait;
use sqlx::{PgPool, Row};

use crate::modules::user_setting::domain::{UserSetting, UserSettingRepository};
use crate::shared::errors::AppError;

/// SQLx/Postgres implementation of [`UserSettingRepository`], targeting the
/// schema defined by the migrations under `databases/postgresql`. Every
/// query filters on `user_id` -- see the trait doc for why.
#[derive(Clone)]
pub struct UserSettingRepositoryPg {
    pool: PgPool,
}

impl UserSettingRepositoryPg {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn map_row(row: &sqlx::postgres::PgRow) -> UserSetting {
        UserSetting {
            id: row.get("id"),
            user_id: row.get("user_id"),
            key: row.get("key"),
            value: row.get("value"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}

#[async_trait]
impl UserSettingRepository for UserSettingRepositoryPg {
    async fn list_for_user(&self, user_id: i32) -> Result<Vec<UserSetting>, AppError> {
        let rows = sqlx::query("SELECT * FROM user_settings WHERE user_id = $1 ORDER BY key")
            .bind(user_id)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.iter().map(Self::map_row).collect())
    }

    async fn find(&self, user_id: i32, key: &str) -> Result<Option<UserSetting>, AppError> {
        let row = sqlx::query("SELECT * FROM user_settings WHERE user_id = $1 AND key = $2")
            .bind(user_id)
            .bind(key)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.as_ref().map(Self::map_row))
    }

    async fn upsert(
        &self,
        user_id: i32,
        key: &str,
        value: Option<&str>,
    ) -> Result<UserSetting, AppError> {
        let row = sqlx::query(
            r#"
            INSERT INTO user_settings (user_id, key, value)
            VALUES ($1, $2, $3)
            ON CONFLICT (user_id, key) DO UPDATE
                SET value = EXCLUDED.value
            RETURNING *
            "#,
        )
        .bind(user_id)
        .bind(key)
        .bind(value)
        .fetch_one(&self.pool)
        .await?;

        Ok(Self::map_row(&row))
    }

    async fn delete(&self, user_id: i32, key: &str) -> Result<(), AppError> {
        sqlx::query("DELETE FROM user_settings WHERE user_id = $1 AND key = $2")
            .bind(user_id)
            .bind(key)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
