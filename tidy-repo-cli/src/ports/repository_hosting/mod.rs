use async_trait::async_trait;

use crate::application::repository::{BranchNameDto, RepositoryUrlDto};

pub mod github;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum AuthenticationCredentialValidity {
    Valid,
    Invalid,
}

#[async_trait]
pub trait RepositoryHostClient {
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
mockall::mock! {
    pub RepositoryHostClient<Err: 'static + Send + Sync, C: 'static + Send + Sync> {}

    #[async_trait::async_trait]
    trait RepositoryHostClient {
        type Err = Err;
        type AuthenticationCredentials = C;

        async fn list_branches(
            &self,
            repository_url: &RepositoryUrlDto,
        ) -> Result<Vec<BranchNameDto>, Err>;

        fn set_authentication_credentials(&mut self, credentials: C);

        async fn validate_authentication_credentials(
            &self,
            credentials: C,
        ) -> Result<AuthenticationCredentialValidity, Err>;
    }
}
