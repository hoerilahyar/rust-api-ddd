use crate::shared::errors::AppError;

#[derive(Debug, thiserror::Error)]
pub enum UserDomainError {
    #[error("invalid email address")]
    InvalidEmail,

    #[error("username must be 3-150 characters and only contain letters, numbers, '.' or '_'")]
    InvalidUsername,

    #[error("user not found")]
    NotFound,

    #[error("email is already registered")]
    EmailTaken,

    #[error("username is already taken")]
    UsernameTaken,
}

impl From<UserDomainError> for AppError {
    fn from(err: UserDomainError) -> Self {
        match err {
            UserDomainError::InvalidEmail | UserDomainError::InvalidUsername => {
                AppError::BadRequest(err.to_string())
            }
            UserDomainError::NotFound => AppError::NotFound(err.to_string()),
            UserDomainError::EmailTaken | UserDomainError::UsernameTaken => {
                AppError::Conflict(err.to_string())
            }
        }
    }
}
