#[cfg(test)]
use crate::domain::repository_host::TestRepositoryHostError;
use crate::ports::repository_hosting::github::GithubClientError;

#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error(transparent)]
    RepositoryHostError(#[from] RepositoryHostError),
}

#[derive(Debug, thiserror::Error)]
pub enum RepositoryHostError {
    #[error(transparent)]
    GithubRepositoryClientError(#[from] GithubClientError),
    #[cfg(test)]
    #[error(transparent)]
    TestRepositoryClientError(#[from] TestRepositoryHostError),
}
