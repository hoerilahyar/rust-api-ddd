use async_trait::async_trait;

use crate::modules::activity_log::domain::{ActivityLog, ActivityLogQuery};
use crate::shared::errors::AppError;

/// Read-only persistence contract for `activity_logs`.
///
/// Activity logs are immutable (append-only). This repository only exposes
/// read operations and must never modify or delete historical records --
/// writes go through `shared::contracts::ActivityRecorder` instead, so this
/// module can never tamper with the trail it's meant to expose (same split
/// as `audit::domain::AuditLogRepository` / `shared::contracts::AuditRecorder`).
#[async_trait]
pub trait ActivityLogRepository: Send + Sync {
    async fn list(&self, query: &ActivityLogQuery) -> Result<(Vec<ActivityLog>, i64), AppError>;

    async fn find_by_id(&self, id: i64) -> Result<Option<ActivityLog>, AppError>;
}
