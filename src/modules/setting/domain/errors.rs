use crate::shared::errors::AppError;

#[derive(Debug, thiserror::Error)]
pub enum SettingDomainError {
    #[error("setting key is required")]
    InvalidKey,

    #[error("setting not found")]
    NotFound,
}

impl From<SettingDomainError> for AppError {
    fn from(err: SettingDomainError) -> Self {
        match err {
            SettingDomainError::InvalidKey => AppError::BadRequest(err.to_string()),
            SettingDomainError::NotFound => AppError::NotFound(err.to_string()),
        }
    }
}
