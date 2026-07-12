use crate::shared::errors::AppError;

#[derive(Debug, thiserror::Error)]
pub enum PermissionDomainError {
    #[error("invalid name")]
    InvalidName,

    #[error("permission not found")]
    NotFound,

    #[error("permission is already registered")]
    PermissionTaken,
}

impl From<PermissionDomainError> for AppError {
    fn from(err: PermissionDomainError) -> Self {
        match err {
            PermissionDomainError::InvalidName => AppError::BadRequest(err.to_string()),
            PermissionDomainError::NotFound => AppError::NotFound(err.to_string()),
            PermissionDomainError::PermissionTaken => AppError::Conflict(err.to_string()),
        }
    }
}
