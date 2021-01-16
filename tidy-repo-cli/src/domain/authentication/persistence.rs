use async_trait::async_trait;

use crate::domain::authentication::GitHubAuthenticationToken;

#[async_trait]
pub trait PersistAuthentication {
    type Err;

    async fn persist_credentials(
        &self,
        credentials: GitHubAuthenticationToken,
    ) -> Result<(), Self::Err>;

    async fn credentials(&self) -> Result<GitHubAuthenticationToken, Self::Err>;
}

#[cfg(test)]
mockall::mock! {
    pub PersistAuthentication<Err: 'static + Send + Sync> {}

    #[async_trait::async_trait]
    pub trait PersistAuthentication {
        type Err = Err;

        async fn persist_credentials(
            &self,
            credentials: GitHubAuthenticationToken,
        ) -> Result<(), Err>;

        async fn credentials(
            &self,
        ) -> Result<GitHubAuthenticationToken, Err>;
    }
}
