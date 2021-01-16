use std::collections::HashMap;

use futures::future::try_join_all;

use crate::application::{repository::RepositoryUrlDto, ApplicationError};
use crate::domain::authentication::{AuthenticationService, GitHubAuthenticationToken};
use crate::domain::count_branches::BranchCounterService;
use crate::domain::error::DomainError;
use crate::domain::repository::{Repository, RepositoryProvider, RepositoryUrl};

pub struct ApplicationService<BranchCounter, GAS, GRP>
where
    BranchCounter: BranchCounterService,
    GAS: AuthenticationService<AuthenticationCredentials = GitHubAuthenticationToken>,
    GRP: RepositoryProvider,
{
    branch_counter_service: BranchCounter,
    github_authentication_service: GAS,
    github_repository_provider: GRP,
}

impl<BranchCounter, GAS, GRP> ApplicationService<BranchCounter, GAS, GRP>
where
    BranchCounter: BranchCounterService,
    GAS: AuthenticationService<AuthenticationCredentials = GitHubAuthenticationToken>,
    GRP: RepositoryProvider,
{
    pub fn new(
        branch_counter_service: BranchCounter,
        github_authentication_service: GAS,
        github_repository_provider: GRP,
    ) -> Self {
        ApplicationService {
            branch_counter_service,
            github_authentication_service,
            github_repository_provider,
        }
    }

    async fn get_repositories(
        &self,
        repository_urls: Vec<RepositoryUrlDto>,
    ) -> Result<Vec<Repository>, DomainError> {
        let domain_urls: Vec<RepositoryUrl> = repository_urls
            .iter()
            .cloned()
            .map(RepositoryUrlDto::into)
            .collect();
        try_join_all(
            domain_urls
                .iter()
                .map(|url| self.github_repository_provider.get_repository(url)),
        )
        .await
        .map_err(DomainError::from)
    }

    pub async fn count_branches_in_repositories(
        &self,
        repository_urls: Vec<RepositoryUrlDto>,
    ) -> Result<HashMap<RepositoryUrlDto, u32>, ApplicationError> {
        let repositories = self.get_repositories(repository_urls).await?;

        Ok(self
            .branch_counter_service
            .count_branches_in_repositories(repositories)
            .iter()
            .map(|(repository, count)| (repository.url().clone().into(), *count))
            .collect())
    }

    pub async fn authenticate_app_with_github(
        &self,
        github_token: String,
    ) -> Result<(), ApplicationError> {
        self.github_authentication_service
            .authenticate(GitHubAuthenticationToken::new(github_token))
            .await
            .map_err(DomainError::from)
            .map_err(ApplicationError::from)
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::eq;
    use spectral::prelude::*;

    use crate::domain::authentication::{AuthenticationError, MockAuthenticationService};
    use crate::domain::count_branches::BranchCounterServiceImpl;
    use crate::domain::repository::Branch;
    use crate::domain::repository::MockRepositoryProvider;
    use crate::utils::test_helpers::async_this;

    use super::*;

    type MockGitHubAuthenticationService = MockAuthenticationService<GitHubAuthenticationToken>;

    fn under_test<BCS: BranchCounterService>(
        branch_counter_service: BCS,
        github_authentication_service: MockGitHubAuthenticationService,
        github_repository_provider: MockRepositoryProvider,
    ) -> ApplicationService<BCS, MockGitHubAuthenticationService, MockRepositoryProvider> {
        ApplicationService::new(
            branch_counter_service,
            github_authentication_service,
            github_repository_provider,
        )
    }

    fn repository_with_branches(n: u32) -> Repository {
        Repository::new(
            RepositoryUrl::new(n.to_string()),
            (0..n).map(|index| Branch::new(index.to_string())).collect(),
        )
    }

    fn prepare_mock_repository_provider(
        mock: &mut MockRepositoryProvider,
        urls_and_branch_counts: Vec<(RepositoryUrl, u32)>,
    ) {
        urls_and_branch_counts
            .iter()
            .cloned()
            .for_each(|(url, count)| {
                mock.expect_get_repository()
                    .with(eq(url))
                    .returning(move |_| Box::pin(async_this(Ok(repository_with_branches(count)))));
            });
    }

    fn to_urls(repository_url_strings: Vec<&str>) -> Vec<RepositoryUrlDto> {
        repository_url_strings
            .iter()
            .map(|s| RepositoryUrlDto::new(s.to_string()))
            .collect()
    }

    fn to_hash_map(list: Vec<(RepositoryUrlDto, u32)>) -> HashMap<RepositoryUrlDto, u32> {
        let mut hash_map = HashMap::new();
        list.iter().for_each(|(url, count)| {
            hash_map.insert(url.clone(), *count);
        });
        hash_map
    }

    #[async_std::test]
    async fn counts_branches_in_list_of_repositories() {
        let branch_counter_service = BranchCounterServiceImpl::new();
        let mock_github_authentication_service = MockGitHubAuthenticationService::default();
        let mut mock_github_repository_provider = MockRepositoryProvider::default();
        prepare_mock_repository_provider(
            &mut mock_github_repository_provider,
            vec![
                (RepositoryUrl::new("1".to_string()), 1),
                (RepositoryUrl::new("2".to_string()), 2),
            ],
        );

        assert_that(
            &under_test(
                branch_counter_service,
                mock_github_authentication_service,
                mock_github_repository_provider,
            )
            .count_branches_in_repositories(to_urls(vec!["1", "2"]))
            .await
            .unwrap(),
        )
        .is_equal_to(&to_hash_map(vec![
            (RepositoryUrlDto::new("1".to_string()), 1u32),
            (RepositoryUrlDto::new("2".to_string()), 2u32),
        ]));
    }

    #[async_std::test]
    async fn authenticates_with_github() {
        let branch_counter_service = BranchCounterServiceImpl::new();
        let mock_github_repository_provider = MockRepositoryProvider::default();
        let mut mock_github_authentication_service = MockGitHubAuthenticationService::default();
        mock_github_authentication_service
            .expect_authenticate()
            .with(eq(GitHubAuthenticationToken::new(
                "credentials".to_string(),
            )))
            .returning(|_| Ok(()));

        assert_that(
            &under_test(
                branch_counter_service,
                mock_github_authentication_service,
                mock_github_repository_provider,
            )
            .authenticate_app_with_github("credentials".to_string())
            .await,
        )
        .is_ok();
    }

    #[async_std::test]
    async fn fails_to_authenticate_with_github_when_persistence_fails() {
        let branch_counter_service = BranchCounterServiceImpl::new();
        let mock_github_repository_provider = MockRepositoryProvider::default();
        let mut mock_github_authentication_service = MockGitHubAuthenticationService::default();
        mock_github_authentication_service
            .expect_authenticate()
            .with(eq(GitHubAuthenticationToken::new(
                "credentials".to_string(),
            )))
            .returning(|_| Err(AuthenticationError::Persistence));

        let result = under_test(
            branch_counter_service,
            mock_github_authentication_service,
            mock_github_repository_provider,
        )
        .authenticate_app_with_github("credentials".to_string())
        .await;

        assert_that(&matches!(
            result.err().unwrap(),
            ApplicationError::Domain(DomainError::Authentication(
                AuthenticationError::Persistence
            ))
        ))
        .is_true();
    }
}
