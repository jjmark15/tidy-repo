pub use branch::*;
pub use github_repository_provider::*;
pub use repository_url::*;

use crate::ports::repository_hosting::adapters::github::{
    GitHubClientError, RepositoryUrlParseError,
};

mod branch;
mod github_repository_provider;
mod repository_url;

impl From<GitHubClientError> for RepositoryProviderError {
    fn from(client_error: GitHubClientError) -> Self {
        match client_error {
            GitHubClientError::ApiUrlParseError(..)
            | GitHubClientError::HttpClientError(..)
            | GitHubClientError::Unexpected
            | GitHubClientError::JsonDeserializationError(..) => {
                RepositoryProviderError::GitHubClient(client_error)
            }
            GitHubClientError::RepositoryNotFound(url) => {
                RepositoryProviderError::RepositoryNotFound(url)
            }
            GitHubClientError::RepositoryUrlParseError(parse_error) => {
                RepositoryProviderError::InvalidUrl(parse_error)
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RepositoryProviderError {
    #[error("GitHub client error occurred ({0})")]
    GitHubClient(GitHubClientError),
    #[error(transparent)]
    InvalidUrl(RepositoryUrlParseError),
    #[error("repository '{0}' not found")]
    RepositoryNotFound(RepositoryUrlDto),
}
