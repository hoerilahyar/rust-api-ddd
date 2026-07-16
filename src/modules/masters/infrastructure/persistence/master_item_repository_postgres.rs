use async_trait::async_trait;
use serde_json::Value;
use sqlx::{PgPool, Row};

use crate::modules::masters::domain::{MasterItem, MasterItemRepository};
use crate::shared::domain::PaginationParams;
use crate::shared::errors::AppError;

#[derive(Clone)]
pub struct MasterItemRepositoryPg {
    pool: PgPool,
}

impl MasterItemRepositoryPg {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn map_row(row: &sqlx::postgres::PgRow) -> MasterItem {
        MasterItem {
            id: row.get("id"),
            group_id: row.get("group_id"),
            code: row.get("code"),
            name: row.get("name"),
            extra: row.get("extra"),
            sort_order: row.get("sort_order"),
            description: row.get("description"),
            is_active: row.get("is_active"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            deleted_at: row.get("deleted_at"),
        }
    }
}

#[async_trait]
impl MasterItemRepository for MasterItemRepositoryPg {
    async fn find_by_id(&self, id: i64) -> Result<Option<MasterItem>, AppError> {
        let row = sqlx::query("SELECT * FROM master_items WHERE id = $1 AND deleted_at IS NULL")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| Self::map_row(&r)))
    }

    async fn find_by_name(&self, name: &str) -> Result<Option<MasterItem>, AppError> {
        let row = sqlx::query("SELECT * FROM master_items WHERE name = $1 AND deleted_at IS NULL")
            .bind(name)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| Self::map_row(&r)))
    }

    async fn find_by_group_and_code(
        &self,
        group_id: i64,
        code: &str,
    ) -> Result<Option<MasterItem>, AppError> {
        let row = sqlx::query(
            "SELECT * FROM master_items WHERE group_id = $1 AND code = $2 AND deleted_at IS NULL",
        )
        .bind(group_id)
        .bind(code)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| Self::map_row(&r)))
    }

    async fn list(
        &self,
        pagination: &PaginationParams,
    ) -> Result<(Vec<MasterItem>, i64), AppError> {
        let (_page, limit) = pagination.normalized();
        let offset = pagination.offset();
        let search = pagination.search.clone().unwrap_or_default();
        let search_pattern = format!("%{search}%");

        let rows = sqlx::query(
            r#"
            SELECT * FROM master_items
            WHERE deleted_at IS NULL
              AND ($3 = '' OR name ILIKE $4)
            ORDER BY sort_order ASC, id DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .bind(&search)
        .bind(&search_pattern)
        .fetch_all(&self.pool)
        .await?;

        let total: i64 = sqlx::query(
            r#"
            SELECT COUNT(*) AS total FROM master_items
            WHERE deleted_at IS NULL
              AND ($1 = '' OR name ILIKE $2)
            "#,
        )
        .bind(&search)
        .bind(&search_pattern)
        .fetch_one(&self.pool)
        .await?
        .get("total");

        let master_items: Vec<MasterItem> = rows.iter().map(Self::map_row).collect();

        Ok((master_items, total))
    }

    async fn create(
        &self,
        group_id: i64,
        code: &str,
        name: &str,
        description: Option<&str>,
        extra: Option<Value>,
        sort_order: Option<i32>,
    ) -> Result<MasterItem, AppError> {
        let row = sqlx::query(
            r#"
            INSERT INTO master_items (group_id, code, name, description, extra, sort_order)
            VALUES ($1, $2, $3, $4, COALESCE($5, '{}'::jsonb), COALESCE($6, 0))
            RETURNING *
            "#,
        )
        .bind(group_id)
        .bind(code)
        .bind(name)
        .bind(description)
        .bind(extra)
        .bind(sort_order)
        .fetch_one(&self.pool)
        .await?;

        Ok(Self::map_row(&row))
    }

    async fn update(
        &self,
        id: i64,
        code: Option<&str>,
        name: Option<&str>,
        description: Option<&str>,
        extra: Option<Value>,
        sort_order: Option<i32>,
        is_active: Option<&bool>,
    ) -> Result<MasterItem, AppError> {
        let row = sqlx::query(
            r#"
            UPDATE master_items
            SET code = COALESCE($2, code),
                name = COALESCE($3, name),
                description = COALESCE($4, description),
                extra = COALESCE($5, extra),
                sort_order = COALESCE($6, sort_order),
                is_active = COALESCE($7, is_active)
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(code)
        .bind(name)
        .bind(description)
        .bind(extra)
        .bind(sort_order)
        .bind(is_active)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("master item not found".to_string()))?;

        Ok(Self::map_row(&row))
    }

    async fn delete(&self, id: i64) -> Result<(), AppError> {
        sqlx::query(
            "UPDATE master_items SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
