use async_trait::async_trait;

pub use github_authentication_service::*;
pub use github_token::*;

use crate::domain::authentication_persistence::AuthenticationPersistenceError;
use crate::domain::repository_host::RepositoryHostError;

mod github_authentication_service;
mod github_token;

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
    pub trait AuthenticationService {
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
    #[error(transparent)]
    PersistenceError(#[from] AuthenticationPersistenceError),
    #[error(transparent)]
    RepositoryHost(#[from] RepositoryHostError),
    #[cfg(test)]
    #[error("test authentication error")]
    TestVariant,
}
