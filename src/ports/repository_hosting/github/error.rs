use crate::ports::repository_hosting::github::parse_repository_url::RepositoryUrlParseError;
use crate::ports::repository_hosting::github::repository::RepositoryUrl;

#[derive(Debug, thiserror::Error)]
pub enum GitHubClientError {
    #[error(transparent)]
    RepositoryUrlParseError(#[from] RepositoryUrlParseError),
    #[error(transparent)]
    HttpClientError(#[from] crate::utils::http::Error),
    #[error("JSON deserialization error: {0}")]
    JsonDeserializationError(#[from] serde_json::Error),
    #[error(transparent)]
    ApiUrlParseError(http_types::url::ParseError),
    #[error("repository '{0}' not found")]
    RepositoryNotFound(RepositoryUrl),
    #[error("unexpected response from GitHub")]
    Unexpected,
}
