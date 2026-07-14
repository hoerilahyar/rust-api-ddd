use std::sync::Arc;

use async_trait::async_trait;

use crate::modules::activity_log::application::service::ActivityLogService;
use crate::modules::activity_log::domain::{ActivityLog, ActivityLogQuery, ActivityLogRepository};
use crate::shared::errors::AppError;

pub struct ActivityLogServiceImpl {
    repo: Arc<dyn ActivityLogRepository>,
}

impl ActivityLogServiceImpl {
    pub fn new(repo: Arc<dyn ActivityLogRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl ActivityLogService for ActivityLogServiceImpl {
    async fn list(&self, query: &ActivityLogQuery) -> Result<(Vec<ActivityLog>, i64), AppError> {
        self.repo.list(query).await
    }

    async fn get_by_id(&self, id: i64) -> Result<ActivityLog, AppError> {
        self.repo
            .find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound("activity log not found".to_string()))
    }
}
