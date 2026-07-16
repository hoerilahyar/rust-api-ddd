use crate::shared::errors::AppError;

#[derive(Debug, thiserror::Error)]
pub enum UserProfileDomainError {
    #[error("profile not found")]
    NotFound,
}

impl From<UserProfileDomainError> for AppError {
    fn from(err: UserProfileDomainError) -> Self {
        match err {
            UserProfileDomainError::NotFound => AppError::NotFound(err.to_string()),
        }
    }
}
