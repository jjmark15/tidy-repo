use async_trait::async_trait;

use crate::domain::authentication::persistence::CredentialRepository;
use crate::domain::authentication::AuthenticationValidity;
use crate::domain::authentication::{
    AuthenticationError, AuthenticationService, GitHubAuthenticationToken,
    RepositoryAuthenticationValidator,
};

#[derive(Debug, Default)]
pub struct GitHubAuthenticationService<AV, PA>
where
    AV: RepositoryAuthenticationValidator,
    PA: CredentialRepository,
{
    authentication_validator: AV,
    authentication_persistence: PA,
}

impl<AV, PA> GitHubAuthenticationService<AV, PA>
where
    AV: RepositoryAuthenticationValidator,
    PA: CredentialRepository,
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
    AV: RepositoryAuthenticationValidator + Send + Sync,
    PA: CredentialRepository + Send + Sync,
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
                .store(credentials)
                .await
                .map_err(|_| AuthenticationError::Persistence),
            AuthenticationValidity::Invalid => Err(AuthenticationError::InvalidCredentials),
        }
    }

    async fn authentication_credentials(
        &self,
    ) -> Result<Self::AuthenticationCredentials, AuthenticationError> {
        self.authentication_persistence
            .get()
            .await
            .map_err(|_| AuthenticationError::Persistence)
    }
}

#[cfg(test)]
mod tests {
    use predicates::ord::eq;
    use spectral::prelude::*;

    use crate::domain::authentication::persistence::MockCredentialRepository;
    use crate::domain::authentication::AuthenticationValidity;
    use crate::domain::authentication::MockRepositoryAuthenticationValidator;

    use super::*;

    type MockRepositoryAuthenticationValidatorAlias = MockRepositoryAuthenticationValidator<()>;
    type MockCredentialRepositoryAlias = MockCredentialRepository<()>;

    fn under_test(
        authentication_validator: MockRepositoryAuthenticationValidatorAlias,
        authentication_persistence: MockCredentialRepositoryAlias,
    ) -> GitHubAuthenticationService<
        MockRepositoryAuthenticationValidatorAlias,
        MockCredentialRepositoryAlias,
    > {
        GitHubAuthenticationService::new(authentication_validator, authentication_persistence)
    }

    fn mock_credential_repository() -> MockCredentialRepositoryAlias {
        MockCredentialRepositoryAlias::default()
    }

    fn mock_authentication_validator() -> MockRepositoryAuthenticationValidatorAlias {
        MockRepositoryAuthenticationValidatorAlias::default()
    }

    #[async_std::test]
    async fn authenticates_valid_credentials() {
        let token = GitHubAuthenticationToken::new("credentials".into());
        let mut mock_credential_repository = mock_credential_repository();
        mock_credential_repository
            .expect_store()
            .with(eq(token.clone()))
            .returning(|_| Ok(()));
        let mut mock_authentication_validator = mock_authentication_validator();
        mock_authentication_validator
            .expect_validate_authentication_credentials()
            .with(eq(token.clone()))
            .returning(|_| Ok(AuthenticationValidity::Valid));

        assert_that(
            &under_test(mock_authentication_validator, mock_credential_repository)
                .authenticate(token)
                .await,
        )
        .is_ok();
    }

    #[async_std::test]
    async fn fails_to_authenticate_invalid_credentials() {
        let token = GitHubAuthenticationToken::new("credentials".into());
        let mut mock_credential_repository = mock_credential_repository();
        mock_credential_repository
            .expect_store()
            .with(eq(token.clone()))
            .returning(|_| Ok(()));
        let mut mock_authentication_validator = mock_authentication_validator();
        mock_authentication_validator
            .expect_validate_authentication_credentials()
            .with(eq(token.clone()))
            .returning(|_| Ok(AuthenticationValidity::Invalid));

        let result = under_test(mock_authentication_validator, mock_credential_repository)
            .authenticate(token)
            .await;

        assert_that(&matches!(result.err().unwrap(), AuthenticationError::InvalidCredentials {..}))
            .is_true();
    }

    #[async_std::test]
    async fn fails_to_authenticate_valid_credentials_when_fails_to_persist() {
        let token = GitHubAuthenticationToken::new("credentials".into());
        let mut mock_credential_repository = mock_credential_repository();
        mock_credential_repository
            .expect_store()
            .with(eq(token.clone()))
            .returning(|_| Err(()));
        let mut mock_authentication_validator = mock_authentication_validator();
        mock_authentication_validator
            .expect_validate_authentication_credentials()
            .with(eq(token.clone()))
            .returning(|_| Ok(AuthenticationValidity::Valid));

        let result = under_test(mock_authentication_validator, mock_credential_repository)
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
        let mut mock_credential_repository = mock_credential_repository();
        mock_credential_repository
            .expect_get()
            .returning(|| Ok(GitHubAuthenticationToken::new("credentials".into())));
        let mock_authentication_validator = mock_authentication_validator();

        assert_that(
            &under_test(mock_authentication_validator, mock_credential_repository)
                .authentication_credentials()
                .await,
        )
        .is_ok();
    }

    #[async_std::test]
    async fn fails_to_return_persisted_authentication_credentials_when_persistence_service_fails() {
        let mut mock_credential_repository = mock_credential_repository();
        mock_credential_repository
            .expect_get()
            .returning(|| Err(()));
        let mock_authentication_validator = mock_authentication_validator();

        let result = under_test(mock_authentication_validator, mock_credential_repository)
            .authentication_credentials()
            .await;

        assert_that(&matches!(
            result.err().unwrap(),
            AuthenticationError::Persistence
        ))
        .is_true();
    }
}
