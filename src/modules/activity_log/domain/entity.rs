use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::shared::contracts::{Activity, MethodRequest, Module};

/// Read-only projection of the `activity_logs` table. Rows are only ever
/// written through `shared::contracts::ActivityRecorder`; this module exists
/// purely to list/inspect what's already there (mirrors `audit::domain::LoginLog`
/// for `user_login_logs`).
#[derive(Debug, Clone, Serialize)]
pub struct ActivityLog {
    pub id: i64,
    pub user_id: Option<i32>,

    pub activity: Activity,
    pub module: Module,

    pub resource_type: Option<String>,
    pub resource_id: Option<String>,

    pub method: MethodRequest,
    pub path: String,

    pub description: Option<String>,

    pub ip_address: Option<String>,
    pub user_agent: Option<String>,

    pub status_code: Option<i16>,

    pub trace_id: Option<Uuid>,

    pub created_at: DateTime<Utc>,
}
