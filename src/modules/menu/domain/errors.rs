use crate::shared::errors::AppError;

#[derive(Debug, thiserror::Error)]
pub enum MenuDomainError {
    #[error("name is required")]
    InvalidName,

    #[error("menu not found")]
    NotFound,

    #[error("parent menu not found")]
    ParentNotFound,

    #[error("slug is already registered")]
    SlugTaken,

    #[error("a menu cannot be moved under itself or one of its own descendants")]
    CircularParent,
}

impl From<MenuDomainError> for AppError {
    fn from(err: MenuDomainError) -> Self {
        match err {
            MenuDomainError::InvalidName => AppError::BadRequest(err.to_string()),
            MenuDomainError::NotFound => AppError::NotFound(err.to_string()),
            MenuDomainError::ParentNotFound => AppError::NotFound(err.to_string()),
            MenuDomainError::SlugTaken => AppError::Conflict(err.to_string()),
            MenuDomainError::CircularParent => AppError::BadRequest(err.to_string()),
        }
    }
}
