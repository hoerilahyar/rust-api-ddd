use async_trait::async_trait;
use sqlx::{PgPool, Row};

use crate::modules::masters::domain::{MasterGroup, MasterGroupRepository, MasterItem};
use crate::modules::masters::infrastructure::persistence::MasterItemRepositoryPg;
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

    async fn load_items(&self, group_id: i64) -> Result<Vec<MasterItem>, AppError> {
        let rows = sqlx::query(
            "SELECT * FROM master_items WHERE group_id = $1 AND deleted_at IS NULL ORDER BY sort_order ASC, id DESC",
        )
        .bind(group_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.iter().map(MasterItemRepositoryPg::map_row).collect())
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
            items: Vec::new(),
        }
    }
}

#[async_trait]
impl MasterGroupRepository for MasterGroupRepositoryPg {
    async fn find_by_id(&self, id: i64) -> Result<Option<MasterGroup>, AppError> {
        let row = sqlx::query("SELECT * FROM master_groups WHERE id = $1 AND deleted_at IS NULL")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        match row {
            Some(r) => {
                let mut group = Self::map_row(&r);
                group.items = self.load_items(group.id).await?;
                Ok(Some(group))
            }
            None => Ok(None),
        }
    }

    async fn find_by_name(&self, name: &str) -> Result<Option<MasterGroup>, AppError> {
        let row = sqlx::query("SELECT * FROM master_groups WHERE name = $1 AND deleted_at IS NULL")
            .bind(name)
            .fetch_optional(&self.pool)
            .await?;

        match row {
            Some(r) => {
                let mut group = Self::map_row(&r);
                group.items = self.load_items(group.id).await?;
                Ok(Some(group))
            }
            None => Ok(None),
        }
    }

    async fn find_by_code(&self, code: &str) -> Result<Option<MasterGroup>, AppError> {
        let row = sqlx::query("SELECT * FROM master_groups WHERE code = $1 AND deleted_at IS NULL")
            .bind(code)
            .fetch_optional(&self.pool)
            .await?;

        match row {
            Some(r) => {
                let mut group = Self::map_row(&r);
                group.items = self.load_items(group.id).await?;
                Ok(Some(group))
            }
            None => Ok(None),
        }
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

        let mut master_groups: Vec<MasterGroup> = rows.iter().map(Self::map_row).collect();

        let group_ids: Vec<i64> = master_groups.iter().map(|g| g.id).collect();

        if !group_ids.is_empty() {
            let item_rows = sqlx::query(
                r#"
            SELECT * FROM master_items
            WHERE group_id = ANY($1) AND deleted_at IS NULL
            ORDER BY sort_order ASC, id DESC
            "#,
            )
            .bind(&group_ids)
            .fetch_all(&self.pool)
            .await?;

            let items: Vec<MasterItem> = item_rows
                .iter()
                .map(MasterItemRepositoryPg::map_row)
                .collect();

            for group in master_groups.iter_mut() {
                group.items = items
                    .iter()
                    .filter(|i| i.group_id == group.id)
                    .cloned()
                    .collect();
            }
        }

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
        id: i64,
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

    async fn delete(&self, id: i64) -> Result<(), AppError> {
        sqlx::query(
            "UPDATE master_groups SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
