use serde::export::PhantomData;

use crate::domain::authentication::persistence::PersistAuthentication;
use crate::ports::persistence::{
    Credentials, GitHubAuthenticationToken as PersistenceGitHubAuthenticationToken, Persist,
    PersistenceError,
};

#[derive(Default)]
pub struct FilesystemAuthenticationPersistenceService<C, P: Persist> {
    credentials_type_marker: PhantomData<C>,
    persistence_service: P,
}

impl<C, P: Persist> FilesystemAuthenticationPersistenceService<C, P> {
    pub fn new(persistence_service: P) -> Self {
        FilesystemAuthenticationPersistenceService {
            credentials_type_marker: PhantomData::default(),
            persistence_service,
        }
    }
}

#[async_trait::async_trait]
impl<C, P> PersistAuthentication for FilesystemAuthenticationPersistenceService<C, P>
where
    C: Sync
        + Send
        + Into<PersistenceGitHubAuthenticationToken>
        + From<PersistenceGitHubAuthenticationToken>,
    P: Persist + Sync + Send,
{
    type AuthenticationCredentials = C;
    type Err = PersistenceError;

    async fn persist_credentials(
        &self,
        credentials: Self::AuthenticationCredentials,
    ) -> Result<(), Self::Err> {
        let credentials_at_rest = Credentials::new(credentials.into());
        self.persistence_service
            .store(credentials_at_rest)
            .await
            .map_err(Into::into)
    }

    async fn credentials(&self) -> Result<Self::AuthenticationCredentials, Self::Err> {
        self.persistence_service
            .load()
            .await
            .map_err(Into::into)
            .map(|credentials_at_rest| credentials_at_rest.github_token().clone().into())
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use mockall::predicate::eq;
    use spectral::prelude::*;

    use crate::ports::persistence::{Credentials, GitHubAuthenticationToken, MockPersist};

    use super::*;

    fn under_test(
        persistence_service: MockPersist,
    ) -> FilesystemAuthenticationPersistenceService<PersistenceGitHubAuthenticationToken, MockPersist>
    {
        FilesystemAuthenticationPersistenceService::new(persistence_service)
    }

    #[derive(Debug, Eq, PartialEq)]
    struct TestAuthenticationCredentials {
        value: String,
    }

    impl TestAuthenticationCredentials {
        pub fn new(value: String) -> Self {
            TestAuthenticationCredentials { value }
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
                .persist_credentials(PersistenceGitHubAuthenticationToken::new(
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
                Ok(Credentials::new(GitHubAuthenticationToken::new(
                    "credentials".to_string(),
                )))
            });

        assert_that(
            &under_test(mock_persistence_service)
                .credentials()
                .await
                .unwrap(),
        )
        .is_equal_to(&PersistenceGitHubAuthenticationToken::new(
            "credentials".to_string(),
        ));
    }
}
