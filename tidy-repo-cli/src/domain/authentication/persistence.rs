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
    #[error("Credential does not exist in storage")]
    CredentialDoesNotExist,
    #[error("Failed to retrieve credential")]
    FailedToGetCredential,
    #[error("Storage contains corrupted data")]
    CorruptData,
    #[error("Failed to store credential")]
    FailedToStoreCredential,
}
