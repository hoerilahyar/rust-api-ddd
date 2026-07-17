use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::{modules::log_audit_trails::domain::entity::AuditTrailLog, shared::errors::AppError};

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

/// Cross-cutting audit contract. Lives in `shared` (not in `auth`) so that
/// any module can record an auditable action against `audit_trail_logs`
/// without depending on the auth module's persistence layer directly.
#[async_trait]
pub trait AuditTrailRecorder: Send + Sync {
    async fn record_audit_trail_log(&self, log: AuditTrailLog) -> Result<(), AppError>;
}
