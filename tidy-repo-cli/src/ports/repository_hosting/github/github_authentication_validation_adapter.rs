use crate::domain::authentication::{
    AuthenticationValidity, GitHubAuthenticationToken, RepositoryAuthenticationValidator,
};
use crate::ports::repository_hosting::github::{
    GitHubAuthenticationToken as RepositoryClientGitHubAuthenticationToken, GitHubClientError,
    RepositoryHostClient,
};
use crate::ports::repository_hosting::AuthenticationCredentialValidity;

pub struct GitHubAuthenticationValidatorAdapter<GC>
where
    GC: RepositoryHostClient<
        Err = GitHubClientError,
        AuthenticationCredentials = RepositoryClientGitHubAuthenticationToken,
    >,
{
    github_client: GC,
}

impl<GC> GitHubAuthenticationValidatorAdapter<GC>
where
    GC: RepositoryHostClient<
        Err = GitHubClientError,
        AuthenticationCredentials = RepositoryClientGitHubAuthenticationToken,
    >,
{
    pub fn new(github_client: GC) -> Self {
        GitHubAuthenticationValidatorAdapter { github_client }
    }
}

#[async_trait::async_trait]
impl<GC> RepositoryAuthenticationValidator for GitHubAuthenticationValidatorAdapter<GC>
where
    GC: RepositoryHostClient<
            Err = GitHubClientError,
            AuthenticationCredentials = RepositoryClientGitHubAuthenticationToken,
        > + Send
        + Sync,
{
    type Err = GitHubClientError;

    async fn validate_authentication_credentials(
        &self,
        credentials: GitHubAuthenticationToken,
    ) -> Result<AuthenticationValidity, Self::Err> {
        let validity = match self
            .github_client
            .validate_authentication_credentials(RepositoryClientGitHubAuthenticationToken::new(
                credentials.value(),
            ))
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
    use crate::ports::repository_hosting::github::MockRepositoryHostClient;

    use super::*;

    type MockRepositoryHostClientAlias =
        MockRepositoryHostClient<GitHubClientError, RepositoryClientGitHubAuthenticationToken>;

    fn under_test(
        github_client: MockRepositoryHostClientAlias,
    ) -> GitHubAuthenticationValidatorAdapter<MockRepositoryHostClientAlias> {
        GitHubAuthenticationValidatorAdapter::new(github_client)
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
