use async_trait::async_trait;
use sqlx::{PgPool, Row};

use crate::modules::menu::domain::{Menu, MenuRepository};
use crate::shared::errors::AppError;

/// SQLx/Postgres implementation of [`MenuRepository`], targeting the schema
/// defined by the migrations under `databases/postgresql`.
#[derive(Clone)]
pub struct MenuRepositoryPg {
    pool: PgPool,
}

impl MenuRepositoryPg {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// N+1 by design, matching the existing `role`/`permission` repositories
    /// (`attach_permissions` is called per row there too) -- menus are a
    /// small dataset, so this stays consistent with the rest of the codebase
    /// rather than introducing a one-off aggregated-join query just here.
    async fn attach_permissions(&self, mut menu: Menu) -> Result<Menu, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT p.name
            FROM permissions p
            INNER JOIN menu_permissions mp ON mp.permission_id = p.id
            WHERE mp.menu_id = $1
            ORDER BY p.name
            "#,
        )
        .bind(menu.id)
        .fetch_all(&self.pool)
        .await?;

        menu.permissions = rows
            .into_iter()
            .map(|r| r.get::<String, _>("name"))
            .collect();
        Ok(menu)
    }

    fn map_row(row: &sqlx::postgres::PgRow) -> Menu {
        Menu {
            id: row.get("id"),
            parent_id: row.get("parent_id"),
            name: row.get("name"),
            slug: row.get("slug"),
            path: row.get("path"),
            icon: row.get("icon"),
            order_index: row.get("order_index"),
            is_active: row.get("is_active"),
            permissions: Vec::new(),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }
    }
}

#[async_trait]
impl MenuRepository for MenuRepositoryPg {
    async fn find_by_id(&self, id: i32) -> Result<Option<Menu>, AppError> {
        let row = sqlx::query("SELECT * FROM menus WHERE id = $1 AND deleted_at IS NULL")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        match row {
            Some(r) => Ok(Some(self.attach_permissions(Self::map_row(&r)).await?)),
            None => Ok(None),
        }
    }

    async fn find_by_slug(&self, slug: &str) -> Result<Option<Menu>, AppError> {
        let row = sqlx::query("SELECT * FROM menus WHERE slug = $1 AND deleted_at IS NULL")
            .bind(slug)
            .fetch_optional(&self.pool)
            .await?;

        match row {
            Some(r) => Ok(Some(self.attach_permissions(Self::map_row(&r)).await?)),
            None => Ok(None),
        }
    }

    async fn list_all(&self) -> Result<Vec<Menu>, AppError> {
        let rows =
            sqlx::query("SELECT * FROM menus WHERE deleted_at IS NULL ORDER BY order_index, id")
                .fetch_all(&self.pool)
                .await?;

        let mut menus = Vec::with_capacity(rows.len());
        for row in rows {
            menus.push(self.attach_permissions(Self::map_row(&row)).await?);
        }
        Ok(menus)
    }

    async fn create(
        &self,
        parent_id: Option<i32>,
        name: &str,
        slug: &str,
        path: Option<&str>,
        icon: Option<&str>,
        order_index: i32,
    ) -> Result<Menu, AppError> {
        let row = sqlx::query(
            r#"
            INSERT INTO menus (parent_id, name, slug, path, icon, order_index)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(parent_id)
        .bind(name)
        .bind(slug)
        .bind(path)
        .bind(icon)
        .bind(order_index)
        .fetch_one(&self.pool)
        .await?;

        Ok(Self::map_row(&row))
    }

    async fn update(
        &self,
        id: i32,
        parent_id: Option<Option<i32>>,
        name: Option<&str>,
        path: Option<&str>,
        icon: Option<Option<&str>>,
        order_index: Option<i32>,
        is_active: Option<bool>,
    ) -> Result<Menu, AppError> {
        let should_reparent = parent_id.is_some();
        let new_parent = parent_id.flatten();

        let should_touch_icon = icon.is_some();
        let new_icon = icon.flatten();
        println!(
            "icon param: should_touch={:?} new_icon={:?}",
            should_touch_icon, new_icon
        );
        let row = sqlx::query(
            r#"
            UPDATE menus
            SET parent_id   = CASE WHEN $2::bool THEN $3::int4 ELSE parent_id END,
                name        = COALESCE($4::text, name),
                path        = COALESCE($5::text, path),
                icon        = CASE WHEN $6::bool THEN $7::text ELSE icon END,
                order_index = COALESCE($8::int4, order_index),
                is_active   = COALESCE($9::bool, is_active)
            WHERE id = $1::int4 AND deleted_at IS NULL
            RETURNING *
            "#,
        )
        .bind(id) // $1
        .bind(should_reparent) // $2
        .bind(new_parent) // $3
        .bind(name) // $4
        .bind(path) // $5
        .bind(should_touch_icon) // $6
        .bind(new_icon) // $7
        .bind(order_index) // $8
        .bind(is_active) // $9
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("menu not found".to_string()))?;

        self.attach_permissions(Self::map_row(&row)).await
    }

    async fn delete(&self, id: i32) -> Result<(), AppError> {
        // Recursive CTE walks the subtree rooted at `id` and soft-deletes
        // every row in it in one statement, so a parent's children never end
        // up orphaned pointing at a deleted row.
        sqlx::query(
            r#"
            WITH RECURSIVE subtree AS (
                SELECT id FROM menus WHERE id = $1 AND deleted_at IS NULL
                UNION ALL
                SELECT m.id
                FROM menus m
                INNER JOIN subtree s ON m.parent_id = s.id
                WHERE m.deleted_at IS NULL
            )
            UPDATE menus
            SET deleted_at = now()
            WHERE id IN (SELECT id FROM subtree)
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn assign_permission(&self, menu_id: i32, permission_id: i32) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO menu_permissions (menu_id, permission_id)
            VALUES ($1, $2)
            ON CONFLICT (menu_id, permission_id) DO NOTHING
            "#,
        )
        .bind(menu_id)
        .bind(permission_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn revoke_permission(&self, menu_id: i32, permission_id: i32) -> Result<(), AppError> {
        sqlx::query("DELETE FROM menu_permissions WHERE menu_id = $1 AND permission_id = $2")
            .bind(menu_id)
            .bind(permission_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn find_permission_by_name(&self, name: &str) -> Result<Option<(i32, String)>, AppError> {
        let row = sqlx::query("SELECT id, name FROM permissions WHERE name = $1")
            .bind(name)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| (r.get("id"), r.get("name"))))
    }
}
