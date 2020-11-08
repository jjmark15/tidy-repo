use async_trait::async_trait;

pub use authenticated::*;
pub use error::*;
pub use unauthenticated::*;

use crate::domain::branch::Branch;
use crate::domain::repository::RepositoryUrl;
use crate::ports::repository_hosting::AuthenticationCredentialValidity;

mod authenticated;
mod error;
mod unauthenticated;

#[async_trait]
pub trait RepositoryHostWrapper {
    type AuthenticationCredentials;

    async fn list_branches(
        &mut self,
        repository_url: &RepositoryUrl,
    ) -> Result<Vec<Branch>, RepositoryHostError>;

    async fn validate_authentication_credentials(
        &self,
        credentials: Self::AuthenticationCredentials,
    ) -> Result<AuthenticationCredentialValidity, RepositoryHostError>;
}

#[cfg(test)]
mockall::mock! {
    pub RepositoryHostWrapper<AC: 'static + Send + Sync> {}

    #[async_trait::async_trait]
    trait RepositoryHostWrapper {
        type AuthenticationCredentials = AC;

        async fn list_branches(
            &mut self,
            repository_url: &RepositoryUrl,
        ) -> Result<Vec<Branch>, RepositoryHostError>;

        async fn validate_authentication_credentials(
            &self,
            credentials: AC,
        ) -> Result<AuthenticationCredentialValidity, RepositoryHostError>;
    }
}
