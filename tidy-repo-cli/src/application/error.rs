use crate::application::repository::RepositoryProviderError;
use crate::domain::error::DomainError;

#[derive(Debug, thiserror::Error)]
pub enum ApplicationError {
    #[error(transparent)]
    Domain(#[from] DomainError),
    #[error(transparent)]
    RepositoryProvider(#[from] RepositoryProviderError),
}
