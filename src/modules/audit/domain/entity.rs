use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::shared::contracts::LoginStatus;

/// Read-only projection of the `user_login_logs` table. Rows are only ever
/// written by `shared::contracts::AuditRecorder` (see the `auth` module);
/// this module exists purely to list/inspect what's already there.
#[derive(Debug, Clone, Serialize)]
pub struct LoginLog {
    pub id: i64,
    pub user_id: Option<i32>,
    pub email_attempted: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub status: LoginStatus,
    pub created_at: DateTime<Utc>,
}
