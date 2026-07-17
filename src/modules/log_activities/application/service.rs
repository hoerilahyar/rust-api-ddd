use async_trait::async_trait;

use crate::modules::log_activities::domain::{ActivityLog, ActivityLogQuery};
use crate::shared::errors::AppError;

#[async_trait]
pub trait ActivityLogService: Send + Sync {
    async fn list(&self, query: &ActivityLogQuery) -> Result<(Vec<ActivityLog>, i64), AppError>;
    async fn get_by_id(&self, id: i64) -> Result<ActivityLog, AppError>;
}
