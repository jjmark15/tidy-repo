use crate::domain::authentication::persistence::PersistAuthentication;
use crate::domain::authentication::GitHubAuthenticationToken;
use crate::ports::persistence::{Credentials, Persist, PersistenceError};

#[derive(Default)]
pub struct FilesystemAuthenticationPersistenceService<P: Persist> {
    persistence_service: P,
}

impl<P: Persist> FilesystemAuthenticationPersistenceService<P> {
    pub fn new(persistence_service: P) -> Self {
        FilesystemAuthenticationPersistenceService {
            persistence_service,
        }
    }
}

#[async_trait::async_trait]
impl<P> PersistAuthentication for FilesystemAuthenticationPersistenceService<P>
where
    P: Persist + Sync + Send,
{
    type Err = PersistenceError;

    async fn persist_credentials(
        &self,
        credentials: GitHubAuthenticationToken,
    ) -> Result<(), Self::Err> {
        let credentials_at_rest = Credentials::new(credentials.value());
        self.persistence_service
            .store(credentials_at_rest)
            .await
            .map_err(Into::into)
    }

    async fn credentials(&self) -> Result<GitHubAuthenticationToken, Self::Err> {
        self.persistence_service
            .load()
            .await
            .map_err(Into::into)
            .map(|credentials_at_rest| {
                GitHubAuthenticationToken::new(credentials_at_rest.github_token().to_string())
            })
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::eq;
    use spectral::prelude::*;

    use crate::domain::authentication::GitHubAuthenticationToken;
    use crate::ports::persistence::{Credentials, MockPersist};

    use super::*;

    fn under_test(
        persistence_service: MockPersist,
    ) -> FilesystemAuthenticationPersistenceService<MockPersist> {
        FilesystemAuthenticationPersistenceService::new(persistence_service)
    }

    #[derive(Debug, Eq, PartialEq)]
    struct TestAuthenticationCredentials {
        value: String,
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
                .persist_credentials(GitHubAuthenticationToken::new("credentials".to_string()))
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
            .returning(|| Ok(Credentials::new("credentials".to_string())));

        assert_that(
            &under_test(mock_persistence_service)
                .credentials()
                .await
                .unwrap(),
        )
        .is_equal_to(&GitHubAuthenticationToken::new("credentials".to_string()));
    }
}
