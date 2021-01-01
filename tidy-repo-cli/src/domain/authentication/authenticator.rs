#[async_trait::async_trait]
pub trait RepositoryAuthenticationValidator {
    type AuthenticationCredentials;
    type Err;

    async fn validate_authentication_credentials(
        &self,
        credentials: Self::AuthenticationCredentials,
    ) -> Result<AuthenticationValidity, Self::Err>;
}

#[cfg(test)]
mockall::mock! {
    pub RepositoryAuthenticationValidator<C: 'static + Send + Sync, Err: 'static + Send + Sync> {}

    #[async_trait::async_trait]
    trait RepositoryAuthenticationValidator {
        type AuthenticationCredentials = C;
        type Err = Err;

        async fn validate_authentication_credentials(
            &self,
            credentials: C,
        ) -> Result<AuthenticationValidity, Err>;
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum AuthenticationValidity {
    Valid,
    Invalid,
}
