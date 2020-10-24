use async_trait::async_trait;
use serde::export::PhantomData;

use crate::ports::persistence::{
    Credentials, GitHubAuthenticationToken as PersistenceGitHubAuthenticationToken, Persist,
    PersistenceError,
};

#[async_trait]
pub trait PersistAuthentication {
    type AuthenticationCredentials;

    async fn persist_credentials(
        &self,
        credentials: Self::AuthenticationCredentials,
    ) -> Result<(), AuthenticationPersistenceError>;

    async fn credentials(
        &self,
    ) -> Result<Self::AuthenticationCredentials, AuthenticationPersistenceError>;
}

#[cfg(test)]
mockall::mock! {
    pub PersistAuthentication<AC: 'static + Send + Sync> {}

    #[async_trait::async_trait]
    pub trait PersistAuthentication {
        type AuthenticationCredentials = AC;

        async fn persist_credentials(
            &self,
            credentials: AC,
        ) -> Result<(), AuthenticationPersistenceError>;

        async fn credentials(
            &self,
        ) -> Result<AC, AuthenticationPersistenceError>;
    }
}

#[derive(Debug, Default)]
pub struct PersistAuthenticationImpl<AC, P: Persist> {
    authentication_token_marker: PhantomData<AC>,
    persistence_service: P,
}

impl<AC, P: Persist> PersistAuthenticationImpl<AC, P> {
    pub fn new(persistence_service: P) -> Self {
        PersistAuthenticationImpl {
            authentication_token_marker: PhantomData::default(),
            persistence_service,
        }
    }
}

#[async_trait]
impl<AC, P> PersistAuthentication for PersistAuthenticationImpl<AC, P>
where
    AC: Send
        + Sync
        + Into<PersistenceGitHubAuthenticationToken>
        + From<PersistenceGitHubAuthenticationToken>,
    P: Persist + Send + Sync,
{
    type AuthenticationCredentials = AC;

    async fn persist_credentials(
        &self,
        credentials: Self::AuthenticationCredentials,
    ) -> Result<(), AuthenticationPersistenceError> {
        let credentials_at_rest = Credentials::new(credentials.into());
        self.persistence_service
            .store(credentials_at_rest)
            .await
            .map_err(Into::into)
    }

    async fn credentials(
        &self,
    ) -> Result<Self::AuthenticationCredentials, AuthenticationPersistenceError> {
        self.persistence_service
            .load()
            .await
            .map_err(Into::into)
            .map(|credentials_at_rest| credentials_at_rest.github_token().clone().into())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AuthenticationPersistenceError {
    #[cfg(test)]
    #[error("test authentication persistence error")]
    TestVariant,
    #[error(transparent)]
    Persistence(#[from] PersistenceError),
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use predicates::ord::eq;
    use spectral::prelude::*;

    use crate::ports::persistence::MockPersist;

    use super::*;

    #[derive(Debug, Eq, PartialEq)]
    struct TestAuthenticationCredentials {
        value: String,
    }

    impl TestAuthenticationCredentials {
        pub fn new(value: String) -> Self {
            TestAuthenticationCredentials { value }
        }
    }

    fn under_test(
        persistence_service: MockPersist,
    ) -> PersistAuthenticationImpl<TestAuthenticationCredentials, MockPersist> {
        PersistAuthenticationImpl::new(persistence_service)
    }

    impl From<()> for AuthenticationPersistenceError {
        fn from(_: ()) -> Self {
            AuthenticationPersistenceError::TestVariant
        }
    }

    impl From<TestAuthenticationCredentials> for PersistenceGitHubAuthenticationToken {
        fn from(credentials: TestAuthenticationCredentials) -> Self {
            PersistenceGitHubAuthenticationToken::from_str(credentials.value.as_str()).unwrap()
        }
    }

    impl From<PersistenceGitHubAuthenticationToken> for TestAuthenticationCredentials {
        fn from(token: PersistenceGitHubAuthenticationToken) -> Self {
            TestAuthenticationCredentials::new(token.value())
        }
    }

    #[async_std::test]
    async fn persists_credentials() {
        let mut mock_persistence_service = MockPersist::default();
        mock_persistence_service
            .expect_store()
            .times(1)
            .with(eq(Credentials::new("credentials".parse().unwrap())))
            .returning(|_| Ok(()));

        assert_that(
            &under_test(mock_persistence_service)
                .persist_credentials(TestAuthenticationCredentials::new(
                    "credentials".to_string(),
                ))
                .await,
        )
        .is_ok();
    }

    #[async_std::test]
    async fn returns_persisted_credentials() {
        let mut mock_persistence_service = MockPersist::default();
        mock_persistence_service
            .expect_load()
            .times(1)
            .returning(|| {
                Ok(Credentials::new(
                    PersistenceGitHubAuthenticationToken::from_str("credentials").unwrap(),
                ))
            });

        assert_that(
            &under_test(mock_persistence_service)
                .credentials()
                .await
                .unwrap(),
        )
        .is_equal_to(&TestAuthenticationCredentials::new(
            "credentials".to_string(),
        ));
    }
}
