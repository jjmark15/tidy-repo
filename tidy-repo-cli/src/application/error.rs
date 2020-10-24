use crate::domain::error::DomainError;

#[derive(Debug, thiserror::Error)]
pub enum ApplicationError {
    #[error(transparent)]
    DomainError(#[from] DomainError),
}
