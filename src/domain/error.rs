use crate::domain::authentication::AuthenticationError;
use crate::domain::repository::RepositoryProviderError;

#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error(transparent)]
    Authentication(#[from] AuthenticationError),
    #[error(transparent)]
    RepositoryProvider(#[from] RepositoryProviderError),
}
