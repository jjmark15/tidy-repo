#[cfg(test)]
use crate::ports::repository_hosting::adapters::TestRepositoryHostError;
use crate::ports::repository_hosting::github::GithubClientError;

#[derive(Debug, thiserror::Error)]
pub enum ApplicationError {
    #[error(transparent)]
    RepositoryClientError(#[from] RepositoryHostError),
}

#[derive(Debug, thiserror::Error)]
pub enum RepositoryHostError {
    #[error(transparent)]
    GithubRepositoryClientError(#[from] GithubClientError),
    #[cfg(test)]
    #[error(transparent)]
    TestRepositoryClientError(#[from] TestRepositoryHostError),
}
