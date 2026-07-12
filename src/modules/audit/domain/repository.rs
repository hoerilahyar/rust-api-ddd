use async_trait::async_trait;

use crate::modules::audit::domain::{LoginLog, LoginLogQuery};
use crate::shared::errors::AppError;

/// Read-only persistence contract for `user_login_logs`. There is
/// intentionally no `create`/`update`/`delete` here -- writes go through
/// `shared::contracts::AuditRecorder` instead, so this module can never
/// tamper with the audit trail it's meant to expose.
#[async_trait]
pub trait AuditLogRepository: Send + Sync {
    async fn list(&self, query: &LoginLogQuery) -> Result<(Vec<LoginLog>, i64), AppError>;
    async fn find_by_id(&self, id: i64) -> Result<Option<LoginLog>, AppError>;
}
