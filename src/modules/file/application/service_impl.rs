use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use axum::body::Bytes;
use chrono::Utc;
use tokio::fs::File;
use uuid::Uuid;

use crate::modules::file::application::service::FileService;
use crate::modules::file::domain::{FileAsset, FileRepository, FileStorage};
use crate::modules::log_audit_trails::domain::AuditTrailLog;
use crate::shared::context::current_request_context;
use crate::shared::contracts::AuditTrailRecorder;
use crate::shared::domain::PaginationParams;
use crate::shared::errors::AppError;

const ENTITY_TYPE: &str = "file";

/// Fires an audit trail write in the background so a slow/unavailable audit
/// sink never blocks or fails the actual file mutation. Errors are logged,
/// not propagated -- consistent with how `activity_recorder` calls are
/// fire-and-forget elsewhere in the codebase.
fn spawn_audit_log(
    audit: Arc<dyn AuditTrailRecorder>,
    actor_id: i32,
    action: &'static str,
    file_id: i64,
    old_values: Option<&FileAsset>,
    new_values: Option<&FileAsset>,
) {
    let ctx = current_request_context();
    let log = AuditTrailLog {
        id: 0,
        user_id: Some(actor_id),
        action: action.to_string(),
        entity_type: ENTITY_TYPE.to_string(),
        entity_id: Some(file_id.to_string()),
        old_values: old_values.and_then(|m| serde_json::to_value(m).ok()),
        new_values: new_values.and_then(|m| serde_json::to_value(m).ok()),
        ip_address: ctx.ip_address,
        user_agent: ctx.user_agent,
        description: Some(format!("file id {file_id}")),
        created_at: Utc::now(),
    };

    tokio::spawn(async move {
        if let Err(err) = audit.record_audit_trail_log(log).await {
            tracing::error!(error = ?err, file_id, action, "failed to record file audit trail log");
        }
    });
}

pub struct FileServiceImpl {
    audit: Arc<dyn AuditTrailRecorder>,
    repo: Arc<dyn FileRepository>,
    storage: Arc<dyn FileStorage>,
    max_upload_bytes: usize,
}

impl FileServiceImpl {
    pub fn new(
        audit: Arc<dyn AuditTrailRecorder>,
        repo: Arc<dyn FileRepository>,
        storage: Arc<dyn FileStorage>,
        max_upload_bytes: usize,
    ) -> Self {
        Self {
            audit,
            repo,
            storage,
            max_upload_bytes,
        }
    }

    /// Keeps only the final path component of whatever the client sent as a
    /// filename, so `original_name` can never smuggle `../` or an absolute
    /// path into a response or a future path-building bug -- `stored_name`
    /// (the only thing ever used to address the disk) is always generated
    /// server-side regardless, this is purely for the display value.
    fn sanitize_original_name(raw: &str) -> String {
        let name = Path::new(raw)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("file");
        let name = if name.trim().is_empty() { "file" } else { name };
        name.chars().take(255).collect()
    }
}

#[async_trait]
impl FileService for FileServiceImpl {
    async fn upload(
        &self,
        uploaded_by: Option<i32>,
        original_name: String,
        mime_type: String,
        bytes: Bytes,
        actor_id: i32,
    ) -> Result<FileAsset, AppError> {
        if bytes.is_empty() {
            return Err(AppError::BadRequest("file is empty".to_string()));
        }
        if bytes.len() > self.max_upload_bytes {
            return Err(AppError::BadRequest(format!(
                "file exceeds the {} byte upload limit",
                self.max_upload_bytes
            )));
        }

        let original_name = Self::sanitize_original_name(&original_name);
        let extension = Path::new(&original_name)
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| format!(".{e}"))
            .unwrap_or_default();
        let uuid = Uuid::new_v4();
        let stored_name = format!("{uuid}{extension}");
        let mime_type = if mime_type.trim().is_empty() {
            "application/octet-stream".to_string()
        } else {
            mime_type
        };
        let size_bytes = bytes.len() as i64;

        let storage_path = self.storage.save(&stored_name, bytes).await?;

        let file = match self
            .repo
            .create(
                uuid,
                &original_name,
                &stored_name,
                &mime_type,
                size_bytes,
                &storage_path,
                uploaded_by,
            )
            .await
        {
            Ok(file) => file,
            Err(err) => {
                if let Err(cleanup_err) = self.storage.delete(&storage_path).await {
                    tracing::error!(
                        error = ?cleanup_err,
                        stored_name,
                        "failed to clean up orphaned upload after a failed insert"
                    );
                }

                return Err(err);
            }
        };

        spawn_audit_log(
            self.audit.clone(),
            actor_id,
            "file.upload",
            file.id,
            None,
            Some(&file),
        );

        Ok(file)
    }

    async fn list(&self, pagination: &PaginationParams) -> Result<(Vec<FileAsset>, i64), AppError> {
        self.repo.list(pagination).await
    }

    async fn get_by_uuid(&self, uuid: Uuid) -> Result<FileAsset, AppError> {
        self.repo
            .find_by_uuid(uuid)
            .await?
            .filter(|f| !f.is_deleted())
            .ok_or_else(|| AppError::NotFound("file not found".to_string()))
    }

    async fn open_for_download(
        &self,
        uuid: Uuid,
        actor_id: i32,
    ) -> Result<(FileAsset, File), AppError> {
        let file = self.get_by_uuid(uuid).await?;
        let handle = self.storage.open(&file.storage_path).await?;

        spawn_audit_log(
            self.audit.clone(),
            actor_id,
            "file.open_download",
            file.id,
            None,
            Some(&file),
        );

        Ok((file, handle))
    }

    async fn delete(&self, uuid: Uuid, actor_id: i32) -> Result<(), AppError> {
        let file = self.get_by_uuid(uuid).await?;
        self.repo.soft_delete(file.id).await?;

        // Best-effort: the metadata row is already soft-deleted (the part
        // that matters for every other endpoint), so a disk hiccup here
        // gets logged rather than surfaced as a failed delete request.
        if let Err(err) = self.storage.delete(&file.storage_path).await {
            tracing::error!(error = ?err, file_uuid = %uuid, "failed to remove file bytes from storage");
        }

        spawn_audit_log(
            self.audit.clone(),
            actor_id,
            "file.delete",
            file.id,
            None,
            Some(&file),
        );

        Ok(())
    }
}
