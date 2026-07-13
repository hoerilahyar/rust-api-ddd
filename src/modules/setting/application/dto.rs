use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::modules::setting::domain::SystemSetting;

#[derive(Debug, Deserialize, Validate)]
pub struct UpsertSettingRequest {
    pub value: Option<String>,

    #[validate(length(max = 255, message = "description is too long"))]
    pub description: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SettingResponse {
    pub id: i32,
    pub key: String,
    pub value: Option<String>,
    pub description: Option<String>,
    pub updated_by: Option<i32>,
    pub updated_at: DateTime<Utc>,
}

impl From<SystemSetting> for SettingResponse {
    fn from(s: SystemSetting) -> Self {
        Self {
            id: s.id,
            key: s.key,
            value: s.value,
            description: s.description,
            updated_by: s.updated_by,
            updated_at: s.updated_at,
        }
    }
}
