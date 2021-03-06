use async_trait::async_trait;

use crate::domain::authentication::credential_repository::CredentialRepository;
use crate::domain::authentication::AuthenticationValidity;
use crate::domain::authentication::{
    AuthenticationError, AuthenticationService, GitHubAuthenticationToken,
    RepositoryCredentialsValidator,
};

#[derive(Debug, Default)]
pub struct GitHubAuthenticationService<AV, CR>
where
    AV: RepositoryCredentialsValidator,
    CR: CredentialRepository,
{
    authentication_validator: AV,
    credential_repository: CR,
}

impl<AV, CR> GitHubAuthenticationService<AV, CR>
where
    AV: RepositoryCredentialsValidator,
    CR: CredentialRepository,
{
    pub fn new(authentication_validator: AV, credential_repository: CR) -> Self {
        GitHubAuthenticationService {
            authentication_validator,
            credential_repository,
        }
    }
}

#[async_trait]
impl<AV, CR> AuthenticationService for GitHubAuthenticationService<AV, CR>
where
    AV: RepositoryCredentialsValidator + Send + Sync,
    CR: CredentialRepository + Send + Sync,
{
    type AuthenticationCredentials = GitHubAuthenticationToken;

    async fn authenticate(
        &self,
        credentials: Self::AuthenticationCredentials,
    ) -> Result<(), AuthenticationError> {
        let validity = self
            .authentication_validator
            .validate(credentials.clone())
            .await
            .map_err(|_| AuthenticationError::Validation)?;

        match validity {
            AuthenticationValidity::Valid => self
                .credential_repository
                .store(credentials)
                .await
                .map_err(AuthenticationError::from),
            AuthenticationValidity::Invalid => Err(AuthenticationError::InvalidCredentials),
        }
    }

    async fn authentication_credentials(
        &self,
    ) -> Result<Self::AuthenticationCredentials, AuthenticationError> {
        self.credential_repository
            .get()
            .await
            .map_err(AuthenticationError::from)
    }
}

#[cfg(test)]
mod tests {
    use predicates::ord::eq;
    use spectral::prelude::*;

    use crate::domain::authentication::credential_repository::{
        CredentialRepositoryError, MockCredentialRepository,
    };
    use crate::domain::authentication::AuthenticationValidity;
    use crate::domain::authentication::MockRepositoryCredentialsValidator;
    use crate::utils::test_helpers::async_this;

    use super::*;

    fn under_test(
        authentication_validator: MockRepositoryCredentialsValidator,
        credential_repository: MockCredentialRepository,
    ) -> GitHubAuthenticationService<MockRepositoryCredentialsValidator, MockCredentialRepository>
    {
        GitHubAuthenticationService::new(authentication_validator, credential_repository)
    }

    fn mock_credential_repository() -> MockCredentialRepository {
        MockCredentialRepository::default()
    }

    fn mock_credentials_validator() -> MockRepositoryCredentialsValidator {
        MockRepositoryCredentialsValidator::default()
    }

    #[async_std::test]
    async fn authenticates_valid_credentials() {
        let token = GitHubAuthenticationToken::new("credentials".into());
        let mut mock_credential_repository = mock_credential_repository();
        mock_credential_repository
            .expect_store()
            .with(eq(token.clone()))
            .returning(|_| Box::pin(async_this(Ok(()))));
        let mut mock_credentials_validator = mock_credentials_validator();
        mock_credentials_validator
            .expect_validate()
            .with(eq(token.clone()))
            .returning(|_| Box::pin(async_this(Ok(AuthenticationValidity::Valid))));

        assert_that(
            &under_test(mock_credentials_validator, mock_credential_repository)
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
            .returning(|_| Box::pin(async_this(Ok(()))));
        let mut mock_credentials_validator = mock_credentials_validator();
        mock_credentials_validator
            .expect_validate()
            .with(eq(token.clone()))
            .returning(|_| Box::pin(async_this(Ok(AuthenticationValidity::Invalid))));

        let result = under_test(mock_credentials_validator, mock_credential_repository)
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
            .returning(|_| {
                Box::pin(async_this(Err(
                    CredentialRepositoryError::FailedToStoreCredential,
                )))
            });
        let mut mock_credentials_validator = mock_credentials_validator();
        mock_credentials_validator
            .expect_validate()
            .with(eq(token.clone()))
            .returning(|_| Box::pin(async_this(Ok(AuthenticationValidity::Valid))));

        let result = under_test(mock_credentials_validator, mock_credential_repository)
            .authenticate(token)
            .await;

        assert_that(&matches!(
            result.err().unwrap(),
            AuthenticationError::Persistence(..)
        ))
        .is_true();
    }

    #[async_std::test]
    async fn returns_persisted_authentication_credentials() {
        let mut mock_credential_repository = mock_credential_repository();
        mock_credential_repository.expect_get().returning(|| {
            Box::pin(async_this(Ok(GitHubAuthenticationToken::new(
                "credentials".into(),
            ))))
        });
        let mock_credentials_validator = mock_credentials_validator();

        assert_that(
            &under_test(mock_credentials_validator, mock_credential_repository)
                .authentication_credentials()
                .await,
        )
        .is_ok();
    }

    #[async_std::test]
    async fn fails_to_return_persisted_authentication_credentials_when_persistence_service_fails() {
        let mut mock_credential_repository = mock_credential_repository();
        mock_credential_repository.expect_get().returning(|| {
            Box::pin(async_this(Err(
                CredentialRepositoryError::FailedToGetCredential,
            )))
        });
        let mock_credentials_validator = mock_credentials_validator();

        let result = under_test(mock_credentials_validator, mock_credential_repository)
            .authentication_credentials()
            .await;

        assert_that(&matches!(
            result.err().unwrap(),
            AuthenticationError::Persistence(..)
        ))
        .is_true();
    }
}
