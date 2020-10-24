use crate::domain::authentication::AuthenticationError;
#[cfg(test)]
use crate::ports::repository_hosting::adapters::TestRepositoryHostError;
use crate::ports::repository_hosting::github::GitHubClientError;

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
