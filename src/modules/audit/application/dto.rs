use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::modules::audit::domain::LoginLog;
use crate::shared::contracts::LoginStatus;

#[derive(Debug, Serialize)]
pub struct LoginLogResponse {
    pub id: i64,
    pub user_id: Option<i32>,
    pub email_attempted: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub status: LoginStatus,
    pub created_at: DateTime<Utc>,
}

impl From<LoginLog> for LoginLogResponse {
    fn from(l: LoginLog) -> Self {
        Self {
            id: l.id,
            user_id: l.user_id,
            email_attempted: l.email_attempted,
            ip_address: l.ip_address,
            user_agent: l.user_agent,
            status: l.status,
            created_at: l.created_at,
        }
    }
}
