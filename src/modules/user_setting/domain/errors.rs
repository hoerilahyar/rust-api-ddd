use crate::shared::errors::AppError;

#[derive(Debug, thiserror::Error)]
pub enum UserSettingDomainError {
    #[error("setting key is required")]
    InvalidKey,

    #[error("setting not found")]
    NotFound,
}

impl From<UserSettingDomainError> for AppError {
    fn from(err: UserSettingDomainError) -> Self {
        match err {
            UserSettingDomainError::InvalidKey => AppError::BadRequest(err.to_string()),
            UserSettingDomainError::NotFound => AppError::NotFound(err.to_string()),
        }
    }
}
