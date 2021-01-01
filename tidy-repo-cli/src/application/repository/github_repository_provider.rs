use serde::export::PhantomData;

use crate::application::repository::{RepositoryProviderError, RepositoryUrlDto};
use crate::domain::authentication::persistence::PersistAuthentication;
use crate::domain::authentication::GitHubAuthenticationToken;
use crate::domain::repository::Branch;
use crate::domain::repository::{Repository, RepositoryProvider, RepositoryUrl};
use crate::domain::value_object::ValueObject;
use crate::ports::repository_hosting::adapters::github::{
    GitHubAuthenticationToken as RepositoryClientGitHubAuthenticationToken, GitHubClientError,
};
use crate::ports::repository_hosting::RepositoryHostClient;

#[derive(Default)]
pub struct GitHubRepositoryProvider<GC, AuthPersistence>
where
    GC: RepositoryHostClient<
        Err = GitHubClientError,
        AuthenticationCredentials = RepositoryClientGitHubAuthenticationToken,
    >,
    AuthPersistence: PersistAuthentication<AuthenticationCredentials = GitHubAuthenticationToken>,
{
    github_client: GC,
    authentication_persistence_type_marker: PhantomData<AuthPersistence>,
}

impl<GC, AuthPersistence> GitHubRepositoryProvider<GC, AuthPersistence>
where
    GC: RepositoryHostClient<
        Err = GitHubClientError,
        AuthenticationCredentials = RepositoryClientGitHubAuthenticationToken,
    >,
    AuthPersistence: PersistAuthentication<AuthenticationCredentials = GitHubAuthenticationToken>,
{
    pub fn new(mut github_client: GC, authentication_persistence_service: AuthPersistence) -> Self {
        Self::authenticate_github_client(&mut github_client, &authentication_persistence_service);

        GitHubRepositoryProvider {
            github_client,
            authentication_persistence_type_marker: PhantomData::default(),
        }
    }

    fn authenticate_github_client(
        github_client: &mut GC,
        authentication_persistence_service: &AuthPersistence,
    ) {
        if let Ok(credentials) =
            async_std::task::block_on(authentication_persistence_service.credentials())
        {
            github_client.set_authentication_credentials(credentials.into());
        }
    }
}

#[async_trait::async_trait]
impl<GC, AuthPersistence> RepositoryProvider for GitHubRepositoryProvider<GC, AuthPersistence>
where
    GC: RepositoryHostClient<
            Err = GitHubClientError,
            AuthenticationCredentials = RepositoryClientGitHubAuthenticationToken,
        > + Sync
        + Send,
    AuthPersistence:
        PersistAuthentication<AuthenticationCredentials = GitHubAuthenticationToken> + Sync + Send,
{
    type Error = RepositoryProviderError;

    async fn get_repository(&self, url: &RepositoryUrl) -> Result<Repository, Self::Error> {
        let url_dto = RepositoryUrlDto::new(url.value());
        let branches = self
            .github_client
            .list_branches(&url_dto)
            .await?
            .iter()
            .map(|branch_dto| Branch::new(branch_dto.value().clone()))
            .collect();
        Ok(Repository::new(url.clone(), branches))
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::eq;
    use spectral::prelude::*;

    use crate::application::repository::BranchNameDto;
    use crate::domain::authentication::persistence::MockPersistAuthentication;
    use crate::ports::repository_hosting::adapters::github::GitHubAuthenticationToken as RepositoryClientGitHubAuthenticationToken;
    use crate::ports::repository_hosting::MockRepositoryHostClient;

    use super::*;

    type MockRepositoryHostClientAlias =
        MockRepositoryHostClient<GitHubClientError, RepositoryClientGitHubAuthenticationToken>;
    type MockPersistAuthenticationAlias = MockPersistAuthentication<GitHubAuthenticationToken, ()>;

    fn under_test(
        repository_host_client: MockRepositoryHostClientAlias,
        authentication_persistence_service: MockPersistAuthenticationAlias,
    ) -> GitHubRepositoryProvider<MockRepositoryHostClientAlias, MockPersistAuthenticationAlias>
    {
        GitHubRepositoryProvider::new(repository_host_client, authentication_persistence_service)
    }

    fn prepare_mock_client_list_branches(
        mock_repository_host: &mut MockRepositoryHostClientAlias,
        url: RepositoryUrlDto,
        branches: Vec<BranchNameDto>,
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

    fn prepare_mock_authentication_persistence_service_to_fail(
        mock_authentication_persistence_service: &mut MockPersistAuthenticationAlias,
    ) {
        mock_authentication_persistence_service
            .expect_credentials()
            .returning(|| Err(()));
    }

    fn prepare_mock_authentication_persistence_service_to_succeed(
        mock_authentication_persistence_service: &mut MockPersistAuthenticationAlias,
        credentials: GitHubAuthenticationToken,
    ) {
        mock_authentication_persistence_service
            .expect_credentials()
            .returning(move || Ok(credentials.clone()));
    }

    #[async_std::test]
    async fn gets_repository_given_valid_url_and_no_credentials_available() {
        let mut mock_repository_host_client = MockRepositoryHostClient::default();
        let mut mock_authentication_persistence_service = MockPersistAuthenticationAlias::default();
        prepare_mock_authentication_persistence_service_to_fail(
            &mut mock_authentication_persistence_service,
        );
        prepare_mock_client_list_branches(
            &mut mock_repository_host_client,
            RepositoryUrlDto::new("url".to_string()),
            vec![BranchNameDto::new("1".to_string())],
        );

        assert_that(
            &under_test(
                mock_repository_host_client,
                mock_authentication_persistence_service,
            )
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
        let mut mock_authentication_persistence_service = MockPersistAuthenticationAlias::default();
        prepare_mock_client_set_authentication(
            &mut mock_repository_host_client,
            RepositoryClientGitHubAuthenticationToken::new("token".to_string()),
        );
        prepare_mock_authentication_persistence_service_to_succeed(
            &mut mock_authentication_persistence_service,
            GitHubAuthenticationToken::new("token".to_string()),
        );

        under_test(
            mock_repository_host_client,
            mock_authentication_persistence_service,
        );
    }
}
