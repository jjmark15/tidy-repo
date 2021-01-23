use async_trait::async_trait;

pub use github_authentication_service::*;
pub use github_token::*;
pub use validator::*;

mod github_authentication_service;
mod github_token;
pub mod persistence;
mod validator;

#[async_trait]
pub trait AuthenticationService {
    type AuthenticationCredentials;

    async fn authenticate(
        &self,
        credentials: Self::AuthenticationCredentials,
    ) -> Result<(), AuthenticationError>;

    async fn authentication_credentials(
        &self,
    ) -> Result<Self::AuthenticationCredentials, AuthenticationError>;
}

#[cfg(test)]
mockall::mock! {
    pub AuthenticationService<AC: 'static + Sync + Send> {}

    #[async_trait]
    impl<AC: 'static + Sync + Send> AuthenticationService for AuthenticationService<AC> {
        type AuthenticationCredentials = AC;

        async fn authenticate(
            &self,
            credentials: AC,
        ) -> Result<(), AuthenticationError>;

        async fn authentication_credentials(
            &self,
        ) -> Result<AC, AuthenticationError>;
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AuthenticationError {
    #[error("no credentials found")]
    NoCredentialsFound,
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error("an error occurred during authentication persistence")]
    Persistence,
    #[error("Could not validate authentication")]
    Validation,
    #[cfg(test)]
    #[error("test authentication error")]
    TestVariant,
}
