use async_trait::async_trait;

use crate::modules::log_audit_auths::domain::{LoginLog, LoginLogQuery};
use crate::shared::errors::AppError;

#[async_trait]
pub trait AuditAuthLogService: Send + Sync {
    async fn list(&self, query: &LoginLogQuery) -> Result<(Vec<LoginLog>, i64), AppError>;
    async fn get_by_id(&self, id: i64) -> Result<LoginLog, AppError>;
}
