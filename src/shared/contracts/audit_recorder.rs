use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::shared::errors::AppError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LoginStatus {
    Success,
    Failed,
}

impl LoginStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            LoginStatus::Success => "success",
            LoginStatus::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone)]
pub struct LoginAttempt {
    pub user_id: Option<i32>,
    pub email_attempted: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub status: LoginStatus,
}

/// Cross-cutting audit contract. Lives in `shared` (not in `auth`) so that
/// any module can record an auditable action against `user_login_logs`
/// without depending on the auth module's persistence layer directly.
#[async_trait]
pub trait AuditRecorder: Send + Sync {
    async fn record_login_attempt(&self, attempt: LoginAttempt) -> Result<(), AppError>;
}
