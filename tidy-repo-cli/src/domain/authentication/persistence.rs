use async_trait::async_trait;

use crate::domain::authentication::GitHubAuthenticationToken;

#[async_trait]
#[cfg_attr(test, mockall::automock)]
pub trait CredentialRepository {
    async fn store(
        &self,
        credentials: GitHubAuthenticationToken,
    ) -> Result<(), CredentialRepositoryError>;

    async fn get(&self) -> Result<GitHubAuthenticationToken, CredentialRepositoryError>;
}

#[derive(Debug, thiserror::Error)]
#[cfg_attr(test, derive(Copy, Clone))]
pub enum CredentialRepositoryError {
    #[error("Credential does not exist in repository")]
    CredentialDoesNotExist,
    #[error("Failed to get credential from repository")]
    FailedToGetCredential,
    #[error("Repository contains corrupted data")]
    CorruptData,
    #[error("Failed to store credential in repository")]
    FailedToStoreCredential,
}
