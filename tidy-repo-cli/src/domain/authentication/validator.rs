use crate::domain::authentication::GitHubAuthenticationToken;

#[async_trait::async_trait]
pub trait RepositoryCredentialsValidator {
    type Err;

    async fn validate(
        &self,
        credentials: GitHubAuthenticationToken,
    ) -> Result<AuthenticationValidity, Self::Err>;
}

#[cfg(test)]
mockall::mock! {
    pub RepositoryCredentialsValidator<Err: 'static + Send + Sync> {}

    #[async_trait::async_trait]
    impl<Err: 'static + Send + Sync> RepositoryCredentialsValidator for RepositoryCredentialsValidator<Err> {
        type Err = Err;

        async fn validate(
            &self,
            credentials: GitHubAuthenticationToken,
        ) -> Result<AuthenticationValidity, Err>;
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum AuthenticationValidity {
    Valid,
    Invalid,
}
