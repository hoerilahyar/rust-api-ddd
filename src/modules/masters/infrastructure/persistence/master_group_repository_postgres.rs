use async_trait::async_trait;
use sqlx::{PgPool, Row};

use crate::modules::masters::domain::{MasterGroup, MasterGroupRepository};
use crate::shared::domain::PaginationParams;
use crate::shared::errors::AppError;

#[derive(Clone)]
pub struct MasterGroupRepositoryPg {
    pool: PgPool,
}

impl MasterGroupRepositoryPg {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn map_row(row: &sqlx::postgres::PgRow) -> MasterGroup {
        MasterGroup {
            id: row.get("id"),
            code: row.get("code"),
            name: row.get("name"),
            description: row.get("description"),
            is_active: row.get("is_active"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            deleted_at: row.get("deleted_at"),
            items: Vec::new(), // populate separately (e.g. join/loader) if needed
        }
    }
}

#[async_trait]
impl MasterGroupRepository for MasterGroupRepositoryPg {
    async fn find_by_id(&self, id: i32) -> Result<Option<MasterGroup>, AppError> {
        let row = sqlx::query("SELECT * FROM master_groups WHERE id = $1 AND deleted_at IS NULL")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| Self::map_row(&r)))
    }

    async fn find_by_name(&self, name: &str) -> Result<Option<MasterGroup>, AppError> {
        let row = sqlx::query("SELECT * FROM master_groups WHERE name = $1 AND deleted_at IS NULL")
            .bind(name)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| Self::map_row(&r)))
    }

    async fn list(
        &self,
        pagination: &PaginationParams,
    ) -> Result<(Vec<MasterGroup>, i64), AppError> {
        let (_page, limit) = pagination.normalized();
        let offset = pagination.offset();
        let search = pagination.search.clone().unwrap_or_default();
        let search_pattern = format!("%{search}%");

        let rows = sqlx::query(
            r#"
            SELECT * FROM master_groups
            WHERE deleted_at IS NULL
              AND ($3 = '' OR name ILIKE $4)
            ORDER BY id DESC
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
            SELECT COUNT(*) AS total FROM master_groups
            WHERE deleted_at IS NULL
              AND ($1 = '' OR name ILIKE $2)
            "#,
        )
        .bind(&search)
        .bind(&search_pattern)
        .fetch_one(&self.pool)
        .await?
        .get("total");

        let master_groups: Vec<MasterGroup> = rows.iter().map(Self::map_row).collect();

        Ok((master_groups, total))
    }

    async fn create(
        &self,
        code: &str,
        name: &str,
        description: Option<&str>,
    ) -> Result<MasterGroup, AppError> {
        let row = sqlx::query(
            r#"
            INSERT INTO master_groups (code, name, description)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
        )
        .bind(code)
        .bind(name)
        .bind(description)
        .fetch_one(&self.pool)
        .await?;

        Ok(Self::map_row(&row))
    }

    async fn update(
        &self,
        id: i32,
        code: Option<&str>,
        name: Option<&str>,
        description: Option<&str>,
        is_active: Option<&bool>,
    ) -> Result<MasterGroup, AppError> {
        let row = sqlx::query(
            r#"
            UPDATE master_groups
            SET code = COALESCE($2, code),
                name = COALESCE($3, name),
                description = COALESCE($4, description),
                is_active = COALESCE($5, is_active)
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(code)
        .bind(name)
        .bind(description)
        .bind(is_active)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("master group not found".to_string()))?;

        Ok(Self::map_row(&row))
    }

    async fn delete(&self, id: i32) -> Result<(), AppError> {
        sqlx::query(
            "UPDATE master_groups SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
