use async_trait::async_trait;

use crate::domain::authentication::GitHubAuthenticationToken;

#[async_trait]
pub trait CredentialRepository {
    type Err;

    async fn store(&self, credentials: GitHubAuthenticationToken) -> Result<(), Self::Err>;

    async fn get(&self) -> Result<GitHubAuthenticationToken, Self::Err>;
}

#[cfg(test)]
mockall::mock! {
    pub CredentialRepository<Err: 'static + Send + Sync> {}

    #[async_trait::async_trait]
    impl<Err: 'static + Send + Sync> CredentialRepository for CredentialRepository<Err> {
        type Err = Err;

        async fn store(
            &self,
            credentials: GitHubAuthenticationToken,
        ) -> Result<(), Err>;

        async fn get(
            &self,
        ) -> Result<GitHubAuthenticationToken, Err>;
    }
}
