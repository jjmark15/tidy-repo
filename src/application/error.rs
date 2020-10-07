#[cfg(test)]
use crate::adapters::repository_client::TestRepositoryClientError;
use crate::ports::repository_client::github::GithubRepositoryClientError;

#[derive(Debug, thiserror::Error)]
pub enum ApplicationError {
    #[error(transparent)]
    RepositoryClientError(#[from] RepositoryClientError),
}

#[derive(Debug, thiserror::Error)]
pub enum RepositoryClientError {
    #[error(transparent)]
    GithubRepositoryClientError(#[from] GithubRepositoryClientError),
    #[cfg(test)]
    #[error(transparent)]
    TestRepositoryClientError(#[from] TestRepositoryClientError),
}
