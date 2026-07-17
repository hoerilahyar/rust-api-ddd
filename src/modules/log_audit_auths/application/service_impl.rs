use std::sync::Arc;

use async_trait::async_trait;

use crate::modules::log_audit_auths::application::service::AuditAuthLogService;
use crate::modules::log_audit_auths::domain::{AuditAuthLogRepository, LoginLog, LoginLogQuery};
use crate::shared::errors::AppError;

pub struct AuditAuthLogServiceImpl {
    repo: Arc<dyn AuditAuthLogRepository>,
}

impl AuditAuthLogServiceImpl {
    pub fn new(repo: Arc<dyn AuditAuthLogRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl AuditAuthLogService for AuditAuthLogServiceImpl {
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
