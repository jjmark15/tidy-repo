use crate::domain::authentication::AuthenticationError;
use crate::domain::repository_host::RepositoryHostError;

#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error(transparent)]
    RepositoryHost(#[from] RepositoryHostError),
    #[error(transparent)]
    Authentication(#[from] AuthenticationError),
}
