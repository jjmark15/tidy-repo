use crate::domain::authentication::{
    AuthenticationValidity, GitHubAuthenticationToken, RepositoryAuthenticationValidator,
};
use crate::ports::repository_hosting::github::{
    GitHubAuthenticationToken as RepositoryClientGitHubAuthenticationToken, GitHubClientError,
};
use crate::ports::repository_hosting::{AuthenticationCredentialValidity, RepositoryHostClient};

pub struct GitHubAuthenticationValidator<GC>
where
    GC: RepositoryHostClient<
        Err = GitHubClientError,
        AuthenticationCredentials = RepositoryClientGitHubAuthenticationToken,
    >,
{
    github_client: GC,
}

impl<GC> GitHubAuthenticationValidator<GC>
where
    GC: RepositoryHostClient<
        Err = GitHubClientError,
        AuthenticationCredentials = RepositoryClientGitHubAuthenticationToken,
    >,
{
    pub fn new(github_client: GC) -> Self {
        GitHubAuthenticationValidator { github_client }
    }
}

#[async_trait::async_trait]
impl<GC> RepositoryAuthenticationValidator for GitHubAuthenticationValidator<GC>
where
    GC: RepositoryHostClient<
            Err = GitHubClientError,
            AuthenticationCredentials = RepositoryClientGitHubAuthenticationToken,
        > + Send
        + Sync,
{
    type AuthenticationCredentials = GitHubAuthenticationToken;
    type Err = GitHubClientError;

    async fn validate_authentication_credentials(
        &self,
        credentials: Self::AuthenticationCredentials,
    ) -> Result<AuthenticationValidity, Self::Err> {
        let validity = match self
            .github_client
            .validate_authentication_credentials(credentials.into())
            .await?
        {
            AuthenticationCredentialValidity::Valid => AuthenticationValidity::Valid,
            AuthenticationCredentialValidity::Invalid => AuthenticationValidity::Invalid,
        };
        Ok(validity)
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::eq;
    use spectral::prelude::*;

    use crate::ports::repository_hosting::github::GitHubAuthenticationToken as RepositoryClientGitHubAuthenticationToken;
    use crate::ports::repository_hosting::MockRepositoryHostClient;

    use super::*;

    type MockRepositoryHostClientAlias =
        MockRepositoryHostClient<GitHubClientError, RepositoryClientGitHubAuthenticationToken>;

    fn under_test(
        github_client: MockRepositoryHostClientAlias,
    ) -> GitHubAuthenticationValidator<MockRepositoryHostClientAlias> {
        GitHubAuthenticationValidator::new(github_client)
    }

    fn prepare_mock_client_validate_authentication_credentials(
        mock_repository_host: &mut MockRepositoryHostClientAlias,
        credentials: RepositoryClientGitHubAuthenticationToken,
        validity: AuthenticationCredentialValidity,
    ) {
        mock_repository_host
            .expect_validate_authentication_credentials()
            .with(eq(credentials))
            .returning(move |_| Ok(validity));
    }

    #[async_std::test]
    async fn validates_valid_authentication_credentials() {
        let mut mock_github_client = MockRepositoryHostClientAlias::default();
        prepare_mock_client_validate_authentication_credentials(
            &mut mock_github_client,
            RepositoryClientGitHubAuthenticationToken::new("token".to_string()),
            AuthenticationCredentialValidity::Valid,
        );

        let validity = under_test(mock_github_client)
            .validate_authentication_credentials(GitHubAuthenticationToken::new(
                "token".to_string(),
            ))
            .await
            .unwrap();

        assert_that(&matches!(validity, AuthenticationValidity::Valid)).is_equal_to(true);
    }

    #[async_std::test]
    async fn validates_invalid_authentication_credentials() {
        let mut mock_github_client = MockRepositoryHostClientAlias::default();
        prepare_mock_client_validate_authentication_credentials(
            &mut mock_github_client,
            RepositoryClientGitHubAuthenticationToken::new("token".to_string()),
            AuthenticationCredentialValidity::Invalid,
        );

        let validity = under_test(mock_github_client)
            .validate_authentication_credentials(GitHubAuthenticationToken::new(
                "token".to_string(),
            ))
            .await
            .unwrap();

        assert_that(&matches!(validity, AuthenticationValidity::Invalid)).is_equal_to(true);
    }
}
