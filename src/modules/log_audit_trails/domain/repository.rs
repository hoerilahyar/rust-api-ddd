use async_trait::async_trait;

use crate::modules::log_audit_trails::domain::{AuditTrailLog, AuditTrailLogQuery};
use crate::shared::errors::AppError;

/// Read-only persistence contract for `audit_trail_logs`. There is
/// intentionally no `create`/`update`/`delete` here -- writes go through
/// `shared::contracts::AuditTrailRecorder` instead, so this module can never
/// tamper with the audit trail it's meant to expose.
#[async_trait]
pub trait AuditTrailLogRepository: Send + Sync {
    async fn list(&self, query: &AuditTrailLogQuery)
        -> Result<(Vec<AuditTrailLog>, i64), AppError>;
    async fn find_by_id(&self, id: i64) -> Result<Option<AuditTrailLog>, AppError>;
}
