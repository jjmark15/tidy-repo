use crate::domain::authentication::AuthenticationError;

#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error(transparent)]
    Authentication(#[from] AuthenticationError),
}
