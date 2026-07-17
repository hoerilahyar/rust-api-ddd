use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;

use crate::modules::log_audit_trails::domain::AuditTrailLog;

#[derive(Debug, Serialize)]
pub struct AuditTrailLogResponse {
    pub id: i64,
    pub user_id: Option<i32>,
    pub action: String,
    pub entity_type: String,
    pub entity_id: Option<String>,
    pub old_values: Option<Value>,
    pub new_values: Option<Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<AuditTrailLog> for AuditTrailLogResponse {
    fn from(l: AuditTrailLog) -> Self {
        Self {
            id: l.id,
            action: l.action,
            entity_type: l.entity_type,
            entity_id: l.entity_id,
            old_values: l.old_values,
            new_values: l.new_values,
            ip_address: l.ip_address,
            user_agent: l.user_agent,
            description: l.description,
            user_id: l.user_id,
            created_at: l.created_at,
        }
    }
}
