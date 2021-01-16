use async_trait::async_trait;

use crate::domain::authentication::persistence::PersistAuthentication;
use crate::domain::authentication::AuthenticationValidity;
use crate::domain::authentication::{
    AuthenticationError, AuthenticationService, GitHubAuthenticationToken,
    RepositoryAuthenticationValidator,
};

#[derive(Debug, Default)]
pub struct GitHubAuthenticationService<AV, PA>
where
    AV: RepositoryAuthenticationValidator<AuthenticationCredentials = GitHubAuthenticationToken>,
    PA: PersistAuthentication,
{
    authentication_validator: AV,
    authentication_persistence: PA,
}

impl<AV, PA> GitHubAuthenticationService<AV, PA>
where
    AV: RepositoryAuthenticationValidator<AuthenticationCredentials = GitHubAuthenticationToken>,
    PA: PersistAuthentication,
{
    pub fn new(authentication_validator: AV, authentication_persistence: PA) -> Self {
        GitHubAuthenticationService {
            authentication_validator,
            authentication_persistence,
        }
    }
}

#[async_trait]
impl<AV, PA> AuthenticationService for GitHubAuthenticationService<AV, PA>
where
    AV: RepositoryAuthenticationValidator<AuthenticationCredentials = GitHubAuthenticationToken>
        + Send
        + Sync,
    PA: PersistAuthentication + Send + Sync,
{
    type AuthenticationCredentials = GitHubAuthenticationToken;

    async fn authenticate(
        &self,
        credentials: Self::AuthenticationCredentials,
    ) -> Result<(), AuthenticationError> {
        let validity = self
            .authentication_validator
            .validate_authentication_credentials(credentials.clone())
            .await
            .map_err(|_| AuthenticationError::Validation)?;

        match validity {
            AuthenticationValidity::Valid => self
                .authentication_persistence
                .persist_credentials(credentials)
                .await
                .map_err(|_| AuthenticationError::Persistence),
            AuthenticationValidity::Invalid => Err(AuthenticationError::InvalidCredentials),
        }
    }

    async fn authentication_credentials(
        &self,
    ) -> Result<Self::AuthenticationCredentials, AuthenticationError> {
        self.authentication_persistence
            .credentials()
            .await
            .map_err(|_| AuthenticationError::Persistence)
    }
}

#[cfg(test)]
mod tests {
    use predicates::ord::eq;
    use spectral::prelude::*;

    use crate::domain::authentication::persistence::MockPersistAuthentication;
    use crate::domain::authentication::AuthenticationValidity;
    use crate::domain::authentication::MockRepositoryAuthenticationValidator;

    use super::*;

    type MockRepositoryAuthenticationValidatorAlias =
        MockRepositoryAuthenticationValidator<GitHubAuthenticationToken, ()>;
    type MockPersistAuthenticationAlias = MockPersistAuthentication<()>;

    fn under_test(
        authentication_validator: MockRepositoryAuthenticationValidatorAlias,
        authentication_persistence: MockPersistAuthenticationAlias,
    ) -> GitHubAuthenticationService<
        MockRepositoryAuthenticationValidatorAlias,
        MockPersistAuthenticationAlias,
    > {
        GitHubAuthenticationService::new(authentication_validator, authentication_persistence)
    }

    fn mock_authentication_persistence() -> MockPersistAuthenticationAlias {
        MockPersistAuthenticationAlias::default()
    }

    fn mock_authentication_validator() -> MockRepositoryAuthenticationValidatorAlias {
        MockRepositoryAuthenticationValidatorAlias::default()
    }

    #[async_std::test]
    async fn authenticates_valid_credentials() {
        let token = GitHubAuthenticationToken::new("credentials".into());
        let mut mock_authentication_persistence = mock_authentication_persistence();
        mock_authentication_persistence
            .expect_persist_credentials()
            .with(eq(token.clone()))
            .returning(|_| Ok(()));
        let mut mock_authentication_validator = mock_authentication_validator();
        mock_authentication_validator
            .expect_validate_authentication_credentials()
            .with(eq(token.clone()))
            .returning(|_| Ok(AuthenticationValidity::Valid));

        assert_that(
            &under_test(
                mock_authentication_validator,
                mock_authentication_persistence,
            )
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
        let mut mock_authentication_validator = mock_authentication_validator();
        mock_authentication_validator
            .expect_validate_authentication_credentials()
            .with(eq(token.clone()))
            .returning(|_| Ok(AuthenticationValidity::Invalid));

        let result = under_test(
            mock_authentication_validator,
            mock_authentication_persistence,
        )
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
            .returning(|_| Err(()));
        let mut mock_authentication_validator = mock_authentication_validator();
        mock_authentication_validator
            .expect_validate_authentication_credentials()
            .with(eq(token.clone()))
            .returning(|_| Ok(AuthenticationValidity::Valid));

        let result = under_test(
            mock_authentication_validator,
            mock_authentication_persistence,
        )
        .authenticate(token)
        .await;

        assert_that(&matches!(
            result.err().unwrap(),
            AuthenticationError::Persistence
        ))
        .is_true();
    }

    #[async_std::test]
    async fn returns_persisted_authentication_credentials() {
        let mut mock_authentication_persistence = mock_authentication_persistence();
        mock_authentication_persistence
            .expect_credentials()
            .returning(|| Ok(GitHubAuthenticationToken::new("credentials".into())));
        let mock_authentication_validator = mock_authentication_validator();

        assert_that(
            &under_test(
                mock_authentication_validator,
                mock_authentication_persistence,
            )
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
            .returning(|| Err(()));
        let mock_authentication_validator = mock_authentication_validator();

        let result = under_test(
            mock_authentication_validator,
            mock_authentication_persistence,
        )
        .authentication_credentials()
        .await;

        assert_that(&matches!(
            result.err().unwrap(),
            AuthenticationError::Persistence
        ))
        .is_true();
    }
}
