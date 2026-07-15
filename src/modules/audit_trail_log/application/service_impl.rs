use std::sync::Arc;

use async_trait::async_trait;

use crate::modules::audit_trail_log::{
    application::service::AuditTrailLogService,
    domain::{AuditTrailLog, AuditTrailLogQuery, AuditTrailLogRepository},
};
use crate::shared::errors::AppError;

pub struct AuditTrailLogServiceImpl {
    repo: Arc<dyn AuditTrailLogRepository>,
}

impl AuditTrailLogServiceImpl {
    pub fn new(repo: Arc<dyn AuditTrailLogRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl AuditTrailLogService for AuditTrailLogServiceImpl {
    async fn list(
        &self,
        query: &AuditTrailLogQuery,
    ) -> Result<(Vec<AuditTrailLog>, i64), AppError> {
        self.repo.list(query).await
    }

    async fn get_by_id(&self, id: i64) -> Result<AuditTrailLog, AppError> {
        self.repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("audit trail log not found".to_string()))
    }
}
