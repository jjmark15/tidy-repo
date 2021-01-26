use crate::domain::repository::{Repository, RepositoryUrl};

#[async_trait::async_trait]
#[cfg_attr(test, mockall::automock)]
pub trait RepositoryProvider {
    async fn get_repository(
        &self,
        url: &RepositoryUrl,
    ) -> Result<Repository, RepositoryProviderError>;
}

#[derive(Debug, thiserror::Error)]
#[error("{msg}")]
pub struct RepositoryProviderError {
    msg: String,
}

impl RepositoryProviderError {
    pub fn new(msg: String) -> Self {
        RepositoryProviderError { msg }
    }
}
