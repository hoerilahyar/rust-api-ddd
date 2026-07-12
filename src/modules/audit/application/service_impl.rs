use std::sync::Arc;

use async_trait::async_trait;

use crate::modules::audit::application::service::AuditLogService;
use crate::modules::audit::domain::{AuditLogRepository, LoginLog, LoginLogQuery};
use crate::shared::errors::AppError;

pub struct AuditLogServiceImpl {
    repo: Arc<dyn AuditLogRepository>,
}

impl AuditLogServiceImpl {
    pub fn new(repo: Arc<dyn AuditLogRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl AuditLogService for AuditLogServiceImpl {
    async fn list(&self, query: &LoginLogQuery) -> Result<(Vec<LoginLog>, i64), AppError> {
        self.repo.list(query).await
    }

    async fn get_by_id(&self, id: i64) -> Result<LoginLog, AppError> {
        self.repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("login log not found".to_string()))
    }
}
