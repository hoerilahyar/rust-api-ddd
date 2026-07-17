use async_trait::async_trait;

use crate::{modules::log_audit_trails::domain::entity::AuditTrailLog, shared::errors::AppError};

/// Cross-cutting audit contract. Lives in `shared` (not in `auth`) so that
/// any module can record an auditable action against `log_audit_trails`
/// without depending on the auth module's persistence layer directly.
#[async_trait]
pub trait AuditTrailRecorder: Send + Sync {
    async fn record_audit_trail_log(&self, log: AuditTrailLog) -> Result<(), AppError>;
}
