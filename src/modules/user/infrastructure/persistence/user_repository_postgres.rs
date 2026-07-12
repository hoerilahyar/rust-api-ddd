use async_trait::async_trait;
use sqlx::{PgPool, Row};

use crate::modules::user::domain::entity::User;
use crate::modules::user::domain::repository::UserRepository;
use crate::shared::domain::PaginationParams;
use crate::shared::errors::AppError;

/// SQLx/Postgres implementation of [`UserRepository`].
///
/// NOTE: this file is named `user_repository_postgres.rs` to match the module
/// skeleton (`pub mod user_repository_postgres;` in `persistence/mod.rs`), but
/// the actual backend is PostgreSQL -- that's what the migrations under
/// `databases/postgresql` target. Rename the module/file to
/// `user_repository_postgres.rs` if you want the naming to match reality.
#[derive(Clone)]
pub struct UserRepositoryPg {
    pool: PgPool,
}

impl UserRepositoryPg {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn attach_roles(&self, mut user: User) -> Result<User, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT r.name
            FROM roles r
            INNER JOIN user_roles ur ON ur.role_id = r.id
            WHERE ur.user_id = $1
            ORDER BY r.name
            "#,
        )
        .bind(user.id)
        .fetch_all(&self.pool)
        .await?;

        user.roles = rows
            .into_iter()
            .map(|r| r.get::<String, _>("name"))
            .collect();
        Ok(user)
    }

    fn map_row(row: &sqlx::postgres::PgRow) -> User {
        User {
            id: row.get("id"),
            name: row.get("name"),
            username: row.get("username"),
            email: row.get("email"),
            password_hash: row.get("password_hash"),
            is_active: row.get("is_active"),
            last_login_at: row.get("last_login_at"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            deleted_at: row.get("deleted_at"),
            roles: Vec::new(),
        }
    }

    async fn permissions_for_user(&self, user_id: i32) -> Result<Vec<String>, AppError> {
        let rows = sqlx::query(
            r#"
            SELECT DISTINCT p.name
            FROM permissions p
            INNER JOIN role_permissions rp ON rp.permission_id = p.id
            INNER JOIN user_roles ur ON ur.role_id = rp.role_id
            WHERE ur.user_id = $1
            ORDER BY p.name
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| r.get::<String, _>("name"))
            .collect())
    }
}

use crate::shared::contracts::{UserAuthProjection, UserReader};

/// Implements the cross-module read contract so `auth` can look up
/// credentials without depending on this module's repository trait/type.
#[async_trait]
impl UserReader for UserRepositoryPg {
    async fn find_for_auth(
        &self,
        identifier: &str,
    ) -> Result<Option<UserAuthProjection>, AppError> {
        let row = sqlx::query(
            "SELECT * FROM users WHERE (email = $1 OR username = $1) AND deleted_at IS NULL",
        )
        .bind(identifier)
        .fetch_optional(&self.pool)
        .await?;

        let Some(row) = row else { return Ok(None) };
        let user = self.attach_roles(Self::map_row(&row)).await?;
        let permissions = self.permissions_for_user(user.id).await?;
        Ok(Some(UserAuthProjection {
            id: user.id,
            name: user.name,
            username: user.username,
            email: user.email,
            password_hash: user.password_hash,
            is_active: user.is_active,
            roles: user.roles,
            permissions,
        }))
    }

    async fn find_by_id(&self, id: i32) -> Result<Option<UserAuthProjection>, AppError> {
        let user = UserRepository::find_by_id(self, id).await?;
        let Some(u) = user else { return Ok(None) };
        let permissions = self.permissions_for_user(u.id).await?;
        Ok(Some(UserAuthProjection {
            id: u.id,
            name: u.name,
            username: u.username,
            email: u.email,
            password_hash: u.password_hash,
            is_active: u.is_active,
            roles: u.roles,
            permissions,
        }))
    }

    async fn touch_last_login(&self, id: i32) -> Result<(), AppError> {
        sqlx::query("UPDATE users SET last_login_at = now() WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn update_password(&self, id: i32, password_hash: &str) -> Result<(), AppError> {
        UserRepository::update_password(self, id, password_hash).await
    }
}

#[async_trait]
impl UserRepository for UserRepositoryPg {
    async fn find_by_id(&self, id: i32) -> Result<Option<User>, AppError> {
        let row = sqlx::query("SELECT * FROM users WHERE id = $1 AND deleted_at IS NULL")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        match row {
            Some(r) => Ok(Some(self.attach_roles(Self::map_row(&r)).await?)),
            None => Ok(None),
        }
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        let row = sqlx::query("SELECT * FROM users WHERE email = $1 AND deleted_at IS NULL")
            .bind(email)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| Self::map_row(&r)))
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<User>, AppError> {
        let row = sqlx::query("SELECT * FROM users WHERE username = $1 AND deleted_at IS NULL")
            .bind(username)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| Self::map_row(&r)))
    }

    async fn list(&self, pagination: &PaginationParams) -> Result<(Vec<User>, i64), AppError> {
        let (page, limit) = pagination.normalized();
        let offset = pagination.offset();
        let search = pagination.search.clone().unwrap_or_default();
        let search_pattern = format!("%{search}%");

        let rows = sqlx::query(
            r#"
            SELECT * FROM users
            WHERE deleted_at IS NULL
              AND ($3 = '' OR name ILIKE $4 OR email ILIKE $4 OR username ILIKE $4)
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
            SELECT COUNT(*) AS total FROM users
            WHERE deleted_at IS NULL
              AND ($1 = '' OR name ILIKE $2 OR email ILIKE $2 OR username ILIKE $2)
            "#,
        )
        .bind(&search)
        .bind(&search_pattern)
        .fetch_one(&self.pool)
        .await?
        .get("total");

        let mut users = Vec::with_capacity(rows.len());
        for row in rows {
            users.push(self.attach_roles(Self::map_row(&row)).await?);
        }

        let _ = page;
        Ok((users, total))
    }

    async fn create(
        &self,
        name: &str,
        username: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<User, AppError> {
        let row = sqlx::query(
            r#"
            INSERT INTO users (name, username, email, password_hash, is_active)
            VALUES ($1, $2, $3, $4, true)
            RETURNING *
            "#,
        )
        .bind(name)
        .bind(username)
        .bind(email)
        .bind(password_hash)
        .fetch_one(&self.pool)
        .await?;

        Ok(Self::map_row(&row))
    }

    async fn update(
        &self,
        id: i32,
        name: Option<&str>,
        email: Option<&str>,
        is_active: Option<bool>,
    ) -> Result<User, AppError> {
        let row = sqlx::query(
            r#"
            UPDATE users
            SET name = COALESCE($2, name),
                email = COALESCE($3, email),
                is_active = COALESCE($4, is_active)
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(name)
        .bind(email)
        .bind(is_active)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("user not found".to_string()))?;

        self.attach_roles(Self::map_row(&row)).await
    }

    async fn update_password(&self, id: i32, password_hash: &str) -> Result<(), AppError> {
        sqlx::query("UPDATE users SET password_hash = $2 WHERE id = $1 AND deleted_at IS NULL")
            .bind(id)
            .bind(password_hash)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn soft_delete(&self, id: i32) -> Result<(), AppError> {
        sqlx::query("UPDATE users SET deleted_at = now() WHERE id = $1 AND deleted_at IS NULL")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn assign_role(
        &self,
        user_id: i32,
        role_id: i32,
        assigned_by: Option<i32>,
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
            INSERT INTO user_roles (user_id, role_id, assigned_by)
            VALUES ($1, $2, $3)
            ON CONFLICT (user_id, role_id) DO NOTHING
            "#,
        )
        .bind(user_id)
        .bind(role_id)
        .bind(assigned_by)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn revoke_role(&self, user_id: i32, role_id: i32) -> Result<(), AppError> {
        sqlx::query("DELETE FROM user_roles WHERE user_id = $1 AND role_id = $2")
            .bind(user_id)
            .bind(role_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn find_role_by_name(&self, name: &str) -> Result<Option<(i32, String)>, AppError> {
        let row = sqlx::query("SELECT id, name FROM roles WHERE name = $1")
            .bind(name)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| (r.get("id"), r.get("name"))))
    }
}
