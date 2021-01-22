use crate::domain::authentication::persistence::CredentialRepository;
use crate::domain::authentication::GitHubAuthenticationToken;
use crate::ports::persistence::{Credentials, Persist, PersistenceError};

#[derive(Default)]
pub struct FilesystemCredentialRepositoryAdapter<P: Persist> {
    persistence_service: P,
}

impl<P: Persist> FilesystemCredentialRepositoryAdapter<P> {
    pub fn new(persistence_service: P) -> Self {
        FilesystemCredentialRepositoryAdapter {
            persistence_service,
        }
    }
}

#[async_trait::async_trait]
impl<P> CredentialRepository for FilesystemCredentialRepositoryAdapter<P>
where
    P: Persist + Sync + Send,
{
    type Err = PersistenceError;

    async fn store(&self, credentials: GitHubAuthenticationToken) -> Result<(), Self::Err> {
        let credentials_at_rest = Credentials::new(credentials.value());
        self.persistence_service
            .store(credentials_at_rest)
            .await
            .map_err(Into::into)
    }

    async fn get(&self) -> Result<GitHubAuthenticationToken, Self::Err> {
        self.persistence_service
            .get()
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
    ) -> FilesystemCredentialRepositoryAdapter<MockPersist> {
        FilesystemCredentialRepositoryAdapter::new(persistence_service)
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
                .store(GitHubAuthenticationToken::new("credentials".to_string()))
                .await,
        )
        .is_ok();
    }

    #[async_std::test]
    async fn returns_persisted_credentials() {
        let mut mock_persistence_service = MockPersist::default();
        mock_persistence_service
            .expect_get()
            .times(1)
            .returning(|| Ok(Credentials::new("credentials".to_string())));

        assert_that(&under_test(mock_persistence_service).get().await.unwrap())
            .is_equal_to(&GitHubAuthenticationToken::new("credentials".to_string()));
    }
}
