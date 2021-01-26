use crate::domain::authentication::GitHubAuthenticationToken;

#[async_trait::async_trait]
#[cfg_attr(test, mockall::automock)]
pub trait RepositoryCredentialsValidator {
    async fn validate(
        &self,
        credentials: GitHubAuthenticationToken,
    ) -> Result<AuthenticationValidity, RepositoryCredentialsValidationError>;
}

#[derive(Debug, thiserror::Error)]
pub enum RepositoryCredentialsValidationError {
    #[error("Failed to validate credentials")]
    FailedToValidate,
}

#[derive(Debug, Eq, PartialEq)]
pub enum AuthenticationValidity {
    Valid,
    Invalid,
}
