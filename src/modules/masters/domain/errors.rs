use crate::shared::errors::AppError;

#[derive(Debug, thiserror::Error)]
pub enum MasterGroupDomainError {
    #[error("invalid name")]
    InvalidName,

    #[error("master group not found")]
    NotFound,

    #[error("master group is already created")]
    MasterGroupTaken,
}

impl From<MasterGroupDomainError> for AppError {
    fn from(err: MasterGroupDomainError) -> Self {
        match err {
            MasterGroupDomainError::InvalidName => AppError::BadRequest(err.to_string()),
            MasterGroupDomainError::NotFound => AppError::NotFound(err.to_string()),
            MasterGroupDomainError::MasterGroupTaken => AppError::Conflict(err.to_string()),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MasterItemDomainError {
    #[error("invalid name")]
    InvalidName,

    #[error("master item not found")]
    NotFound,

    #[error("master item is already created")]
    MasterItemTaken,
}

impl From<MasterItemDomainError> for AppError {
    fn from(err: MasterItemDomainError) -> Self {
        match err {
            MasterItemDomainError::InvalidName => AppError::BadRequest(err.to_string()),
            MasterItemDomainError::NotFound => AppError::NotFound(err.to_string()),
            MasterItemDomainError::MasterItemTaken => AppError::Conflict(err.to_string()),
        }
    }
}
