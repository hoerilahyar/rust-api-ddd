use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;

/// Read-only projection of the `audit_trail_logs` table. Rows are only ever
/// written by `shared::contracts::AuditTrailRecorder`
/// this module exists purely to list/inspect what's already there.
#[derive(Debug, Clone, Serialize)]
pub struct AuditTrailLog {
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
