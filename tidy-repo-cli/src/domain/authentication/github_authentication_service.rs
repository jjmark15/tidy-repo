use async_trait::async_trait;

use crate::domain::authentication::persistence::PersistAuthentication;
use crate::domain::authentication::{
    AuthenticationError, AuthenticationService, GitHubAuthenticationToken,
};
use crate::domain::repository_host::RepositoryHostWrapper;
use crate::ports::repository_hosting::AuthenticationCredentialValidity;

#[derive(Debug, Default)]
pub struct GitHubAuthenticationService<RH, PA>
where
    RH: RepositoryHostWrapper<AuthenticationCredentials = GitHubAuthenticationToken>,
    PA: PersistAuthentication<AuthenticationCredentials = GitHubAuthenticationToken>,
{
    repository_host: RH,
    authentication_persistence: PA,
}

impl<RH, PA> GitHubAuthenticationService<RH, PA>
where
    RH: RepositoryHostWrapper<AuthenticationCredentials = GitHubAuthenticationToken>,
    PA: PersistAuthentication<AuthenticationCredentials = GitHubAuthenticationToken>,
{
    pub fn new(repository_host: RH, authentication_persistence: PA) -> Self {
        GitHubAuthenticationService {
            repository_host,
            authentication_persistence,
        }
    }
}

#[async_trait]
impl<RH, PA> AuthenticationService for GitHubAuthenticationService<RH, PA>
where
    RH: RepositoryHostWrapper<AuthenticationCredentials = GitHubAuthenticationToken> + Send + Sync,
    PA: PersistAuthentication<AuthenticationCredentials = GitHubAuthenticationToken> + Send + Sync,
{
    type AuthenticationCredentials = GitHubAuthenticationToken;

    async fn authenticate(
        &self,
        credentials: Self::AuthenticationCredentials,
    ) -> Result<(), AuthenticationError> {
        let validity = self
            .repository_host
            .validate_authentication_credentials(credentials.clone())
            .await
            .map_err(AuthenticationError::from)?;

        match validity {
            AuthenticationCredentialValidity::Valid => self
                .authentication_persistence
                .persist_credentials(credentials)
                .await
                .map_err(Into::into),
            AuthenticationCredentialValidity::Invalid => {
                Err(AuthenticationError::InvalidCredentials)
            }
        }
    }

    async fn authentication_credentials(
        &self,
    ) -> Result<Self::AuthenticationCredentials, AuthenticationError> {
        self.authentication_persistence
            .credentials()
            .await
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use predicates::ord::eq;
    use spectral::prelude::*;

    use crate::domain::authentication::persistence::{
        AuthenticationPersistenceError, MockPersistAuthentication,
    };
    use crate::domain::repository_host::MockRepositoryHostWrapper;
    use crate::ports::repository_hosting::AuthenticationCredentialValidity;

    use super::*;

    fn under_test(
        repository_host: MockRepositoryHostWrapper<GitHubAuthenticationToken>,
        authentication_persistence: MockPersistAuthentication<GitHubAuthenticationToken>,
    ) -> GitHubAuthenticationService<
        MockRepositoryHostWrapper<GitHubAuthenticationToken>,
        MockPersistAuthentication<GitHubAuthenticationToken>,
    > {
        GitHubAuthenticationService::new(repository_host, authentication_persistence)
    }

    fn mock_authentication_persistence() -> MockPersistAuthentication<GitHubAuthenticationToken> {
        MockPersistAuthentication::default()
    }

    fn mock_repository_host() -> MockRepositoryHostWrapper<GitHubAuthenticationToken> {
        MockRepositoryHostWrapper::default()
    }

    #[async_std::test]
    async fn authenticates_valid_credentials() {
        let token = GitHubAuthenticationToken::new("credentials".into());
        let mut mock_authentication_persistence = mock_authentication_persistence();
        mock_authentication_persistence
            .expect_persist_credentials()
            .with(eq(token.clone()))
            .returning(|_| Ok(()));
        let mut mock_repository_host = mock_repository_host();
        mock_repository_host
            .expect_validate_authentication_credentials()
            .with(eq(token.clone()))
            .returning(|_| Ok(AuthenticationCredentialValidity::Valid));

        assert_that(
            &under_test(mock_repository_host, mock_authentication_persistence)
                .authenticate(token)
                .await,
        )
        .is_ok();
    }

    #[async_std::test]
    async fn fails_to_authenticate_invalid_credentials() {
        let token = GitHubAuthenticationToken::new("credentials".into());
        let mut mock_authentication_persistence = mock_authentication_persistence();
        mock_authentication_persistence
            .expect_persist_credentials()
            .with(eq(token.clone()))
            .returning(|_| Ok(()));
        let mut mock_repository_host = mock_repository_host();
        mock_repository_host
            .expect_validate_authentication_credentials()
            .with(eq(token.clone()))
            .returning(|_| Ok(AuthenticationCredentialValidity::Invalid));

        let result = under_test(mock_repository_host, mock_authentication_persistence)
            .authenticate(token)
            .await;

        assert_that(&matches!(result.err().unwrap(), AuthenticationError::InvalidCredentials {..}))
            .is_true();
    }

    #[async_std::test]
    async fn fails_to_authenticate_valid_credentials_when_fails_to_persist() {
        let token = GitHubAuthenticationToken::new("credentials".into());
        let mut mock_authentication_persistence = mock_authentication_persistence();
        mock_authentication_persistence
            .expect_persist_credentials()
            .with(eq(token.clone()))
            .returning(|_| Err(AuthenticationPersistenceError::TestVariant));
        let mut mock_repository_host = mock_repository_host();
        mock_repository_host
            .expect_validate_authentication_credentials()
            .with(eq(token.clone()))
            .returning(|_| Ok(AuthenticationCredentialValidity::Valid));

        let result = under_test(mock_repository_host, mock_authentication_persistence)
            .authenticate(token)
            .await;

        assert_that(&matches!(result.err().unwrap(), AuthenticationError::PersistenceError {..}))
            .is_true();
    }

    #[async_std::test]
    async fn returns_persisted_authentication_credentials() {
        let mut mock_authentication_persistence = mock_authentication_persistence();
        mock_authentication_persistence
            .expect_credentials()
            .returning(|| Ok(GitHubAuthenticationToken::new("credentials".into())));
        let mock_repository_host = mock_repository_host();

        assert_that(
            &under_test(mock_repository_host, mock_authentication_persistence)
                .authentication_credentials()
                .await,
        )
        .is_ok();
    }

    #[async_std::test]
    async fn fails_to_return_persisted_authentication_credentials_when_persistence_service_fails() {
        let mut mock_authentication_persistence = mock_authentication_persistence();
        mock_authentication_persistence
            .expect_credentials()
            .returning(|| Err(AuthenticationPersistenceError::TestVariant));
        let mock_repository_host = mock_repository_host();

        let result = under_test(mock_repository_host, mock_authentication_persistence)
            .authentication_credentials()
            .await;

        assert_that(&matches!(result.err().unwrap(), AuthenticationError::PersistenceError {..}))
            .is_true();
    }
}
