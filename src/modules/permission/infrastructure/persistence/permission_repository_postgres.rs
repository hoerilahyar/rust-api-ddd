use async_trait::async_trait;
use sqlx::{PgPool, Row};

use crate::modules::permission::domain::{Permission, PermissionRepository};
use crate::shared::domain::PaginationParams;
use crate::shared::errors::AppError;

/// SQLx/Postgres implementation of [`PermissionRepository`], targeting the
/// schema defined by the migrations under `databases/postgresql`.
#[derive(Clone)]
pub struct PermissionRepositoryPg {
    pool: PgPool,
}

impl PermissionRepositoryPg {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn map_row(row: &sqlx::postgres::PgRow) -> Permission {
        Permission {
            id: row.get("id"),
            name: row.get("name"),
            description: row.get("description"),
            created_at: row.get("created_at"),
        }
    }
}

#[async_trait]
impl PermissionRepository for PermissionRepositoryPg {
    async fn find_by_id(&self, id: i32) -> Result<Option<Permission>, AppError> {
        let row = sqlx::query("SELECT * FROM permissions WHERE id = $1 ")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        match row {
            Some(r) => Ok(Some(Self::map_row(&r))),
            None => Ok(None),
        }
    }
    async fn find_by_name(&self, name: &str) -> Result<Option<Permission>, AppError> {
        let row = sqlx::query("SELECT * FROM permissions WHERE name = $1 ")
            .bind(name)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| Self::map_row(&r)))
    }

    async fn find_many_by_ids(&self, ids: &[i32]) -> Result<Vec<Permission>, AppError> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let rows = sqlx::query("SELECT * FROM permissions WHERE id = ANY($1)")
            .bind(ids)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.iter().map(Self::map_row).collect())
    }

    async fn list(
        &self,
        pagination: &PaginationParams,
    ) -> Result<(Vec<Permission>, i64), AppError> {
        let (page, limit) = pagination.normalized();
        let offset = pagination.offset();
        let search = pagination.search.clone().unwrap_or_default();
        let search_pattern = format!("%{search}%");

        let rows = sqlx::query(
            r#"
            SELECT * FROM permissions
            WHERE ($3 = '' OR name ILIKE $4 )
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
            SELECT COUNT(*) AS total FROM permissions
            WHERE ($1 = '' OR name ILIKE $2)
            "#,
        )
        .bind(&search)
        .bind(&search_pattern)
        .fetch_one(&self.pool)
        .await?
        .get("total");

        let mut permissions = Vec::with_capacity(rows.len());
        for row in rows {
            permissions.push(Self::map_row(&row));
        }

        let _ = page;
        Ok((permissions, total))
    }

    async fn create(&self, name: &str, description: Option<&str>) -> Result<Permission, AppError> {
        let row = sqlx::query(
            r#"
            INSERT INTO permissions (name, description)
            VALUES ($1, $2)
            RETURNING *
            "#,
        )
        .bind(name)
        .bind(description)
        .fetch_one(&self.pool)
        .await?;

        Ok(Self::map_row(&row))
    }

    async fn update(
        &self,
        id: i32,
        name: Option<&str>,
        description: Option<&str>,
    ) -> Result<Permission, AppError> {
        let row = sqlx::query(
            r#"
            UPDATE permissions
            SET name = COALESCE($2, name),
                description = COALESCE($3, description)
            WHERE id = $1 
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(description)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("permission not found".to_string()))?;

        Ok(Self::map_row(&row))
    }

    async fn delete(&self, id: i32) -> Result<(), AppError> {
        sqlx::query("DELETE FROM permissions WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
