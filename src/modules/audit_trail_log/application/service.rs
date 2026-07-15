use async_trait::async_trait;

use crate::{
    modules::audit_trail_log::domain::{AuditTrailLog, AuditTrailLogQuery},
    shared::errors::AppError,
};

#[async_trait]
pub trait AuditTrailLogService: Send + Sync {
    async fn list(&self, query: &AuditTrailLogQuery)
        -> Result<(Vec<AuditTrailLog>, i64), AppError>;
    async fn get_by_id(&self, id: i64) -> Result<AuditTrailLog, AppError>;
}
