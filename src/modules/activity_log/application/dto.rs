use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::modules::activity_log::domain::ActivityLog;
use crate::shared::contracts::{Activity, MethodRequest, Module};

#[derive(Debug, Serialize)]
pub struct ActivityLogResponse {
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

impl From<ActivityLog> for ActivityLogResponse {
    fn from(a: ActivityLog) -> Self {
        Self {
            id: a.id,
            user_id: a.user_id,
            activity: a.activity,
            module: a.module,
            resource_type: a.resource_type,
            resource_id: a.resource_id,
            method: a.method,
            path: a.path,
            description: a.description,
            ip_address: a.ip_address,
            user_agent: a.user_agent,
            status_code: a.status_code,
            trace_id: a.trace_id,
            created_at: a.created_at,
        }
    }
}
