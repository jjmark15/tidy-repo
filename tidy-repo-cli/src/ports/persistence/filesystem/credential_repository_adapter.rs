use futures::io::ErrorKind;

use crate::domain::authentication::persistence::{CredentialRepository, CredentialRepositoryError};
use crate::domain::authentication::GitHubAuthenticationToken;
use crate::ports::persistence::filesystem::{
    FileSystemCredentialsPersistence, FileSystemPersistenceError,
};
use crate::ports::persistence::Credentials;

#[derive(Default)]
pub struct FilesystemCredentialRepositoryAdapter<P: FileSystemCredentialsPersistence> {
    persistence_service: P,
}

impl<P: FileSystemCredentialsPersistence> FilesystemCredentialRepositoryAdapter<P> {
    pub fn new(persistence_service: P) -> Self {
        FilesystemCredentialRepositoryAdapter {
            persistence_service,
        }
    }
}

#[async_trait::async_trait]
impl<P> CredentialRepository for FilesystemCredentialRepositoryAdapter<P>
where
    P: FileSystemCredentialsPersistence + Sync + Send,
{
    async fn store(
        &self,
        credentials: GitHubAuthenticationToken,
    ) -> Result<(), CredentialRepositoryError> {
        let credentials_at_rest = Credentials::new(credentials.value());
        self.persistence_service
            .store(credentials_at_rest)
            .await
            .map_err(map_filesystem_error_when_storing)
    }

    async fn get(&self) -> Result<GitHubAuthenticationToken, CredentialRepositoryError> {
        self.persistence_service
            .get()
            .await
            .map_err(map_filesystem_error_when_getting)
            .map(|credentials_at_rest| {
                GitHubAuthenticationToken::new(credentials_at_rest.github_token().to_string())
            })
    }
}

fn map_filesystem_error_when_storing(
    error: FileSystemPersistenceError,
) -> CredentialRepositoryError {
    match error {
        FileSystemPersistenceError::Environment(_)
        | FileSystemPersistenceError::IO(_)
        | FileSystemPersistenceError::Serialization(_) => {
            CredentialRepositoryError::FailedToStoreCredential
        }
    }
}

fn map_filesystem_error_when_getting(
    error: FileSystemPersistenceError,
) -> CredentialRepositoryError {
    match error {
        FileSystemPersistenceError::Environment(_) => {
            CredentialRepositoryError::FailedToGetCredential
        }
        FileSystemPersistenceError::IO(e) => match e.kind() {
            ErrorKind::NotFound => CredentialRepositoryError::CredentialDoesNotExist,
            ErrorKind::PermissionDenied => CredentialRepositoryError::FailedToGetCredential,
            ErrorKind::InvalidData => CredentialRepositoryError::CorruptData,
            _ => CredentialRepositoryError::FailedToGetCredential,
        },
        FileSystemPersistenceError::Serialization(_) => CredentialRepositoryError::CorruptData,
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::eq;
    use spectral::prelude::*;

    use crate::domain::authentication::GitHubAuthenticationToken;
    use crate::ports::persistence::filesystem::MockFileSystemCredentialsPersistence;
    use crate::ports::persistence::Credentials;

    use super::*;

    fn under_test(
        persistence_service: MockFileSystemCredentialsPersistence,
    ) -> FilesystemCredentialRepositoryAdapter<MockFileSystemCredentialsPersistence> {
        FilesystemCredentialRepositoryAdapter::new(persistence_service)
    }

    #[derive(Debug, Eq, PartialEq)]
    struct TestAuthenticationCredentials {
        value: String,
    }

    #[async_std::test]
    async fn persists_credentials() {
        let mut mock_persistence_service = MockFileSystemCredentialsPersistence::default();
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
        let mut mock_persistence_service = MockFileSystemCredentialsPersistence::default();
        mock_persistence_service
            .expect_get()
            .times(1)
            .returning(|| Ok(Credentials::new("credentials".to_string())));

        assert_that(&under_test(mock_persistence_service).get().await.unwrap())
            .is_equal_to(&GitHubAuthenticationToken::new("credentials".to_string()));
    }
}
