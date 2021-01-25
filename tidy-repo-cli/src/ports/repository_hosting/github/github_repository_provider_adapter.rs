use std::marker::PhantomData;

use crate::domain::authentication::credential_repository::CredentialRepository;
use crate::domain::repository::{Branch, RepositoryProviderError};
use crate::domain::repository::{Repository, RepositoryProvider, RepositoryUrl};
use crate::domain::value_object::ValueObject;
use crate::ports::repository_hosting::github::error::GitHubClientError;
use crate::ports::repository_hosting::github::repository::RepositoryUrl as RepositoryClientRepositoryUrl;
use crate::ports::repository_hosting::github::{
    authentication_token::GitHubAuthenticationToken as RepositoryClientGitHubAuthenticationToken,
    RepositoryHostClient, RepositoryUrlParseError,
};

#[derive(Default)]
pub struct GitHubRepositoryProviderAdapter<GC, CR>
where
    GC: RepositoryHostClient<
        Err = GitHubClientError,
        AuthenticationCredentials = RepositoryClientGitHubAuthenticationToken,
    >,
    CR: CredentialRepository,
{
    github_client: GC,
    credential_repository_type_marker: PhantomData<CR>,
}

impl<GC, CR> GitHubRepositoryProviderAdapter<GC, CR>
where
    GC: RepositoryHostClient<
        Err = GitHubClientError,
        AuthenticationCredentials = RepositoryClientGitHubAuthenticationToken,
    >,
    CR: CredentialRepository,
{
    pub fn new(mut github_client: GC, credential_repository: CR) -> Self {
        Self::authenticate_github_client(&mut github_client, &credential_repository);

        GitHubRepositoryProviderAdapter {
            github_client,
            credential_repository_type_marker: PhantomData::default(),
        }
    }

    fn authenticate_github_client(github_client: &mut GC, credential_repository: &CR) {
        if let Ok(credentials) = async_std::task::block_on(credential_repository.get()) {
            github_client.set_authentication_credentials(
                RepositoryClientGitHubAuthenticationToken::new(credentials.value()),
            );
        }
    }
}

#[async_trait::async_trait]
impl<GC, CR> RepositoryProvider for GitHubRepositoryProviderAdapter<GC, CR>
where
    GC: RepositoryHostClient<
            Err = GitHubClientError,
            AuthenticationCredentials = RepositoryClientGitHubAuthenticationToken,
        > + Sync
        + Send,
    CR: CredentialRepository + Sync + Send,
{
    async fn get_repository(
        &self,
        url: &RepositoryUrl,
    ) -> Result<Repository, RepositoryProviderError> {
        let url_dto = RepositoryClientRepositoryUrl::new(url.value().clone());
        let branches = self
            .github_client
            .list_branches(&url_dto)
            .await
            .map_err(|err| RepositoryProviderError::from(GitHubRepositoryProviderError::from(err)))?
            .iter()
            .map(|branch_dto| Branch::new(branch_dto.value().clone()))
            .collect();
        Ok(Repository::new(url.clone(), branches))
    }
}

impl From<GitHubClientError> for GitHubRepositoryProviderError {
    fn from(client_error: GitHubClientError) -> Self {
        match client_error {
            GitHubClientError::ApiUrlParseError(..)
            | GitHubClientError::HttpClientError(..)
            | GitHubClientError::Unexpected
            | GitHubClientError::JsonDeserializationError(..) => {
                GitHubRepositoryProviderError::GitHubClient(client_error)
            }
            GitHubClientError::RepositoryNotFound(url) => {
                GitHubRepositoryProviderError::RepositoryNotFound(url)
            }
            GitHubClientError::RepositoryUrlParseError(parse_error) => {
                GitHubRepositoryProviderError::InvalidUrl(parse_error)
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GitHubRepositoryProviderError {
    #[error("GitHub client error occurred ({0})")]
    GitHubClient(GitHubClientError),
    #[error(transparent)]
    InvalidUrl(RepositoryUrlParseError),
    #[error("repository '{0}' not found")]
    RepositoryNotFound(RepositoryClientRepositoryUrl),
}

impl From<GitHubRepositoryProviderError> for RepositoryProviderError {
    fn from(port_error: GitHubRepositoryProviderError) -> Self {
        RepositoryProviderError::new(format!("{}", port_error))
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::eq;
    use spectral::prelude::*;

    use crate::domain::authentication::credential_repository::{
        CredentialRepositoryError, MockCredentialRepository,
    };
    use crate::domain::authentication::GitHubAuthenticationToken;
    use crate::ports::repository_hosting::github::authentication_token::GitHubAuthenticationToken as RepositoryClientGitHubAuthenticationToken;
    use crate::ports::repository_hosting::github::repository::BranchName;
    use crate::ports::repository_hosting::github::MockRepositoryHostClient;
    use crate::utils::test_helpers::async_this;

    use super::*;

    type MockRepositoryHostClientAlias =
        MockRepositoryHostClient<GitHubClientError, RepositoryClientGitHubAuthenticationToken>;

    fn under_test(
        repository_host_client: MockRepositoryHostClientAlias,
        credential_repository: MockCredentialRepository,
    ) -> GitHubRepositoryProviderAdapter<MockRepositoryHostClientAlias, MockCredentialRepository>
    {
        GitHubRepositoryProviderAdapter::new(repository_host_client, credential_repository)
    }

    fn prepare_mock_client_list_branches(
        mock_repository_host: &mut MockRepositoryHostClientAlias,
        url: RepositoryClientRepositoryUrl,
        branches: Vec<BranchName>,
    ) {
        mock_repository_host
            .expect_list_branches()
            .with(eq(url))
            .returning(move |_| Ok(branches.clone()));
    }

    fn prepare_mock_client_set_authentication(
        mock_repository_host: &mut MockRepositoryHostClientAlias,
        credentials: RepositoryClientGitHubAuthenticationToken,
    ) {
        mock_repository_host
            .expect_set_authentication_credentials()
            .with(eq(credentials))
            .once()
            .return_const(());
    }

    fn prepare_mock_credential_repository_to_fail(
        mock_credential_repository: &mut MockCredentialRepository,
        error: CredentialRepositoryError,
    ) {
        mock_credential_repository
            .expect_get()
            .returning(move || Box::pin(async_this(Err(error))));
    }

    fn prepare_mock_credential_repository_to_succeed(
        mock_credential_repository: &mut MockCredentialRepository,
        credentials: GitHubAuthenticationToken,
    ) {
        mock_credential_repository
            .expect_get()
            .returning(move || Box::pin(async_this(Ok(credentials.clone()))));
    }

    #[async_std::test]
    async fn gets_repository_given_valid_url_and_no_credentials_available() {
        let mut mock_repository_host_client = MockRepositoryHostClient::default();
        let mut mock_credential_repository = MockCredentialRepository::default();
        prepare_mock_credential_repository_to_fail(
            &mut mock_credential_repository,
            CredentialRepositoryError::CredentialDoesNotExist,
        );
        prepare_mock_client_list_branches(
            &mut mock_repository_host_client,
            RepositoryClientRepositoryUrl::new("url".to_string()),
            vec![BranchName::new("1".to_string())],
        );

        assert_that(
            &under_test(mock_repository_host_client, mock_credential_repository)
                .get_repository(&RepositoryUrl::new("url".to_string()))
                .await
                .unwrap(),
        )
        .is_equal_to(&Repository::new(
            RepositoryUrl::new("url".to_string()),
            vec![Branch::new("1".to_string())],
        ));
    }

    #[async_std::test]
    async fn authenticates_client_when_credentials_are_available() {
        let mut mock_repository_host_client = MockRepositoryHostClient::default();
        let mut mock_credential_repository = MockCredentialRepository::default();
        prepare_mock_client_set_authentication(
            &mut mock_repository_host_client,
            RepositoryClientGitHubAuthenticationToken::new("token".to_string()),
        );
        prepare_mock_credential_repository_to_succeed(
            &mut mock_credential_repository,
            GitHubAuthenticationToken::new("token".to_string()),
        );

        under_test(mock_repository_host_client, mock_credential_repository);
    }
}
