use crate::shared::errors::AppError;

#[derive(Debug, thiserror::Error)]
pub enum AuthDomainError {
    #[error("invalid email/username or password")]
    InvalidCredentials,

    #[error("account is inactive")]
    AccountInactive,

    #[error("refresh token is invalid or has expired")]
    InvalidRefreshToken,

    #[error("password reset token is invalid or has expired")]
    InvalidResetToken,
}

impl From<AuthDomainError> for AppError {
    fn from(err: AuthDomainError) -> Self {
        match err {
            AuthDomainError::InvalidCredentials => AppError::Unauthorized(err.to_string()),
            AuthDomainError::AccountInactive => AppError::Forbidden(err.to_string()),
            AuthDomainError::InvalidRefreshToken => AppError::Unauthorized(err.to_string()),
            AuthDomainError::InvalidResetToken => AppError::Unauthorized(err.to_string()),
        }
    }
}
