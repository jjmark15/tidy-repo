use crate::domain::authentication::GitHubAuthenticationToken;

#[async_trait::async_trait]
pub trait RepositoryAuthenticationValidator {
    type Err;

    async fn validate_authentication_credentials(
        &self,
        credentials: GitHubAuthenticationToken,
    ) -> Result<AuthenticationValidity, Self::Err>;
}

#[cfg(test)]
mockall::mock! {
    pub RepositoryAuthenticationValidator<Err: 'static + Send + Sync> {}

    #[async_trait::async_trait]
    impl<Err: 'static + Send + Sync> RepositoryAuthenticationValidator for RepositoryAuthenticationValidator<Err> {
        type Err = Err;

        async fn validate_authentication_credentials(
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
