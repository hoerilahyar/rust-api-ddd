use async_trait::async_trait;
use sqlx::{PgPool, Row};

use crate::modules::role::domain::{Role, RoleRepository};
use crate::shared::domain::PaginationParams;
use crate::shared::errors::AppError;

/// SQLx/Postgres implementation of [`RoleRepository`], targeting the schema
/// defined by the migrations under `databases/postgresql`.
#[derive(Clone)]
pub struct RoleRepositoryPg {
    pool: PgPool,
}

impl RoleRepositoryPg {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn attach_permissions(&self, mut role: Role) -> Result<Role, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT p.name
            FROM permissions p
            INNER JOIN role_permissions rp ON rp.permission_id = p.id
            WHERE rp.role_id = $1
            ORDER BY p.name
            "#,
        )
        .bind(role.id)
        .fetch_all(&self.pool)
        .await?;

        role.permissions = rows
            .into_iter()
            .map(|r| r.get::<String, _>("name"))
            .collect();
        Ok(role)
    }

    fn map_row(row: &sqlx::postgres::PgRow) -> Role {
        Role {
            id: row.get("id"),
            name: row.get("name"),
            description: row.get("description"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            permissions: Vec::new(),
        }
    }
}

#[async_trait]
impl RoleRepository for RoleRepositoryPg {
    async fn find_by_id(&self, id: i32) -> Result<Option<Role>, AppError> {
        let row = sqlx::query("SELECT * FROM roles WHERE id = $1 ")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        match row {
            Some(r) => Ok(Some(self.attach_permissions(Self::map_row(&r)).await?)),
            None => Ok(None),
        }
    }
    async fn find_by_name(&self, name: &str) -> Result<Option<Role>, AppError> {
        let row = sqlx::query("SELECT * FROM roles WHERE name = $1 ")
            .bind(name)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| Self::map_row(&r)))
    }

    async fn list(&self, pagination: &PaginationParams) -> Result<(Vec<Role>, i64), AppError> {
        let (page, limit) = pagination.normalized();
        let offset = pagination.offset();
        let search = pagination.search.clone().unwrap_or_default();
        let search_pattern = format!("%{search}%");

        let rows = sqlx::query(
            r#"
            SELECT * FROM roles
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
            SELECT COUNT(*) AS total FROM roles
            WHERE ($1 = '' OR name ILIKE $2)
            "#,
        )
        .bind(&search)
        .bind(&search_pattern)
        .fetch_one(&self.pool)
        .await?
        .get("total");

        let mut roles = Vec::with_capacity(rows.len());
        for row in rows {
            roles.push(self.attach_permissions(Self::map_row(&row)).await?);
        }

        let _ = page;
        Ok((roles, total))
    }

    async fn create(&self, name: &str, description: Option<&str>) -> Result<Role, AppError> {
        let row = sqlx::query(
            r#"
            INSERT INTO roles (name, description)
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
    ) -> Result<Role, AppError> {
        let row = sqlx::query(
            r#"
            UPDATE roles
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
        .ok_or_else(|| AppError::NotFound("role not found".to_string()))?;

        self.attach_permissions(Self::map_row(&row)).await
    }

    async fn delete(&self, id: i32) -> Result<(), AppError> {
        sqlx::query("DELETE FROM roles WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn assign_permission(&self, role_id: i32, permission_id: i32) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO role_permissions (role_id, permission_id)
            VALUES ($1, $2)
            ON CONFLICT (role_id, permission_id) DO NOTHING
            "#,
        )
        .bind(role_id)
        .bind(permission_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
    async fn revoke_permission(&self, role_id: i32, permission_id: i32) -> Result<(), AppError> {
        sqlx::query("DELETE FROM role_permissions WHERE role_id = $1 AND permission_id = $2")
            .bind(role_id)
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
