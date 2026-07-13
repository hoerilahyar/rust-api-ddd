use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::modules::user_setting::domain::UserSetting;

#[derive(Debug, Deserialize, Validate)]
pub struct UpsertUserSettingRequest {
    pub value: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UserSettingResponse {
    pub key: String,
    pub value: Option<String>,
    pub updated_at: DateTime<Utc>,
}

impl From<UserSetting> for UserSettingResponse {
    fn from(s: UserSetting) -> Self {
        Self {
            key: s.key,
            value: s.value,
            updated_at: s.updated_at,
        }
    }
}
