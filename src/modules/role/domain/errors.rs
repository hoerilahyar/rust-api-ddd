use crate::shared::errors::AppError;

#[derive(Debug, thiserror::Error)]
pub enum RoleDomainError {
    #[error("invalid name")]
    InvalidName,

    #[error("role not found")]
    NotFound,

    #[error("role is already registered")]
    RoleTaken,
}

impl From<RoleDomainError> for AppError {
    fn from(err: RoleDomainError) -> Self {
        match err {
            RoleDomainError::InvalidName => AppError::BadRequest(err.to_string()),
            RoleDomainError::NotFound => AppError::NotFound(err.to_string()),
            RoleDomainError::RoleTaken => AppError::Conflict(err.to_string()),
        }
    }
}
