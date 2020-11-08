use crate::domain::authentication::AuthenticationError;
use crate::ports::repository_hosting::adapters::github::GitHubClientError;
#[cfg(test)]
use crate::ports::repository_hosting::TestRepositoryHostError;

#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error(transparent)]
    RepositoryHost(#[from] RepositoryHostError),
    #[error(transparent)]
    Authentication(#[from] AuthenticationError),
}

#[derive(Debug, thiserror::Error)]
pub enum RepositoryHostError {
    #[error(transparent)]
    GitHubClient(#[from] GitHubClientError),
    #[cfg(test)]
    #[error(transparent)]
    TestRepositoryHost(#[from] TestRepositoryHostError),
}
