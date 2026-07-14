use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::modules::file::domain::{FileAsset, FileRepository};
use crate::shared::domain::PaginationParams;
use crate::shared::errors::AppError;

#[derive(Clone)]
pub struct FileRepositoryPg {
    pool: PgPool,
}

impl FileRepositoryPg {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn map_row(row: &sqlx::postgres::PgRow) -> FileAsset {
        FileAsset {
            id: row.get("id"),
            uuid: row.get("uuid"),
            original_name: row.get("original_name"),
            stored_name: row.get("stored_name"),
            mime_type: row.get("mime_type"),
            size_bytes: row.get("size_bytes"),
            storage_path: row.get("storage_path"),
            uploaded_by: row.get("uploaded_by"),
            created_at: row.get("created_at"),
            deleted_at: row.get("deleted_at"),
        }
    }
}

#[async_trait]
impl FileRepository for FileRepositoryPg {
    async fn create(
        &self,
        uuid: Uuid,
        original_name: &str,
        stored_name: &str,
        mime_type: &str,
        size_bytes: i64,
        storage_path: &str,
        uploaded_by: Option<i32>,
    ) -> Result<FileAsset, AppError> {
        let row = sqlx::query(
            r#"
            INSERT INTO files (
                uuid, original_name, stored_name, mime_type,
                size_bytes, storage_path, uploaded_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
        )
        .bind(uuid)
        .bind(original_name)
        .bind(stored_name)
        .bind(mime_type)
        .bind(size_bytes)
        .bind(storage_path)
        .bind(uploaded_by)
        .fetch_one(&self.pool)
        .await?;

        Ok(Self::map_row(&row))
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<FileAsset>, AppError> {
        let row = sqlx::query("SELECT * FROM files WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| Self::map_row(&r)))
    }

    async fn find_by_uuid(&self, uuid: Uuid) -> Result<Option<FileAsset>, AppError> {
        let row = sqlx::query("SELECT * FROM files WHERE uuid = $1")
            .bind(uuid)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| Self::map_row(&r)))
    }

    async fn list(&self, pagination: &PaginationParams) -> Result<(Vec<FileAsset>, i64), AppError> {
        let (_page, limit) = pagination.normalized();
        let offset = pagination.offset();
        let search = pagination.search.clone().unwrap_or_default();
        let search_pattern = format!("%{search}%");

        let rows = sqlx::query(
            r#"
            SELECT *
            FROM files
            WHERE deleted_at IS NULL
              AND ($1 = '' OR original_name ILIKE $2)
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(&search)
        .bind(&search_pattern)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        let total: i64 = sqlx::query(
            r#"
            SELECT COUNT(*) AS total
            FROM files
            WHERE deleted_at IS NULL
              AND ($1 = '' OR original_name ILIKE $2)
            "#,
        )
        .bind(&search)
        .bind(&search_pattern)
        .fetch_one(&self.pool)
        .await?
        .get("total");

        let files = rows.iter().map(Self::map_row).collect();
        Ok((files, total))
    }

    async fn soft_delete(&self, id: i64) -> Result<(), AppError> {
        sqlx::query("UPDATE files SET deleted_at = now() WHERE id = $1 AND deleted_at IS NULL")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
