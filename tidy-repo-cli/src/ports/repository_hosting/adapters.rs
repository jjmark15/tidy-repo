use async_trait::async_trait;

use crate::application::{BranchNameDto, RepositoryUrlDto};
use crate::ports::repository_hosting::AuthenticationCredentialValidity;

#[cfg_attr(test, mockall::automock(
    type Err = TestRepositoryHostError;
    type AuthenticationCredentials = String;
))]
#[async_trait]
pub trait RepositoryHost {
    type Err;
    type AuthenticationCredentials;

    async fn list_branches(
        &self,
        repository_url: &RepositoryUrlDto,
    ) -> Result<Vec<BranchNameDto>, Self::Err>;

    fn set_authentication_credentials(&mut self, credentials: Self::AuthenticationCredentials);

    async fn validate_authentication_credentials(
        &self,
        credentials: Self::AuthenticationCredentials,
    ) -> Result<AuthenticationCredentialValidity, Self::Err>;
}

#[cfg(test)]
#[derive(Debug, thiserror::Error)]
#[error("Repository client error occurred")]
pub struct TestRepositoryHostError;
