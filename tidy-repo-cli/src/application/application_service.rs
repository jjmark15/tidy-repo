use std::collections::HashMap;
use std::iter::FromIterator;

use futures::future::try_join_all;

use crate::application::{ApplicationError, RepositoryUrlDto};
use crate::domain::authentication::AuthenticationService;
use crate::domain::authentication::GitHubAuthenticationToken as DomainCliGitHubAuthenticationToken;
use crate::domain::count_branches::BranchCounterService;
use crate::domain::error::DomainError;
use crate::ports::cli::GitHubAuthenticationToken as CliGitHubAuthenticationToken;

pub struct ApplicationService<BranchCounter, GAS>
where
    BranchCounter: BranchCounterService,
    GAS: AuthenticationService<AuthenticationCredentials = DomainCliGitHubAuthenticationToken>,
{
    branch_counter_service: BranchCounter,
    github_authentication_service: GAS,
}

impl<BranchCounter, GAS> ApplicationService<BranchCounter, GAS>
where
    BranchCounter: BranchCounterService,
    GAS: AuthenticationService<AuthenticationCredentials = DomainCliGitHubAuthenticationToken>,
{
    pub fn new(branch_counter_service: BranchCounter, github_authentication_service: GAS) -> Self {
        ApplicationService {
            branch_counter_service,
            github_authentication_service,
        }
    }

    pub async fn count_branches_in_repositories(
        &self,
        repository_urls: Vec<RepositoryUrlDto>,
    ) -> Result<HashMap<RepositoryUrlDto, u32>, ApplicationError> {
        Ok(HashMap::from_iter(
            try_join_all(
                repository_urls
                    .iter()
                    .map(|url| self.join_url_with_count(url.clone())),
            )
            .await?,
        ))
    }

    pub async fn authenticate_app_with_github(
        &self,
        github_token: CliGitHubAuthenticationToken,
    ) -> Result<(), ApplicationError> {
        self.github_authentication_service
            .authenticate(DomainCliGitHubAuthenticationToken::new(
                github_token.value().to_string(),
            ))
            .await
            .map_err(DomainError::from)
            .map_err(ApplicationError::from)
    }

    async fn join_url_with_count(
        &self,
        url: RepositoryUrlDto,
    ) -> Result<(RepositoryUrlDto, u32), DomainError> {
        let count = self
            .branch_counter_service
            .count_branches(url.clone().into())
            .await?;
        Ok((url, count))
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::eq;
    use spectral::prelude::*;

    use crate::domain::authentication::persistence::AuthenticationPersistenceError;
    use crate::domain::authentication::{AuthenticationError, MockAuthenticationService};
    use crate::domain::count_branches::MockBranchCounterService;
    use crate::domain::repository::RepositoryUrl;

    use super::*;

    type MockGitHubAuthenticationService =
        MockAuthenticationService<DomainCliGitHubAuthenticationToken>;

    fn under_test(
        branch_counter_service: MockBranchCounterService,
        github_authentication_service: MockGitHubAuthenticationService,
    ) -> ApplicationService<MockBranchCounterService, MockGitHubAuthenticationService> {
        ApplicationService::new(branch_counter_service, github_authentication_service)
    }

    fn mock_branch_counter_service() -> MockBranchCounterService {
        MockBranchCounterService::default()
    }

    fn prepare_mock_branch_counter_service(
        mock_branch_counter_service: &mut MockBranchCounterService,
        url: RepositoryUrl,
        count: u32,
    ) {
        async fn async_this<T>(arg: T) -> T {
            arg
        }

        mock_branch_counter_service
            .expect_count_branches()
            .with(eq(url))
            .returning(move |_| Box::pin(async_this(Ok(count))));
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
        let mut mock_branch_counter_service = mock_branch_counter_service();
        prepare_mock_branch_counter_service(
            &mut mock_branch_counter_service,
            RepositoryUrl::new("1".to_string()),
            1,
        );
        prepare_mock_branch_counter_service(
            &mut mock_branch_counter_service,
            RepositoryUrl::new("2".to_string()),
            2,
        );
        let mock_github_authentication_service = MockGitHubAuthenticationService::default();

        assert_that(
            &under_test(
                mock_branch_counter_service,
                mock_github_authentication_service,
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
        let github_credentials = CliGitHubAuthenticationToken::new("credentials".to_string());
        let mock_branch_counter_service = mock_branch_counter_service();
        let mut mock_github_authentication_service = MockGitHubAuthenticationService::default();
        mock_github_authentication_service
            .expect_authenticate()
            .with(eq(DomainCliGitHubAuthenticationToken::new(
                "credentials".to_string(),
            )))
            .returning(|_| Ok(()));

        assert_that(
            &under_test(
                mock_branch_counter_service,
                mock_github_authentication_service,
            )
            .authenticate_app_with_github(github_credentials)
            .await,
        )
        .is_ok();
    }

    #[async_std::test]
    async fn fails_to_authenticate_with_github_when_persistence_fails() {
        let github_credentials = CliGitHubAuthenticationToken::new("credentials".to_string());
        let mock_branch_counter_service = mock_branch_counter_service();
        let mut mock_github_authentication_service = MockGitHubAuthenticationService::default();
        mock_github_authentication_service
            .expect_authenticate()
            .with(eq(DomainCliGitHubAuthenticationToken::new(
                "credentials".to_string(),
            )))
            .returning(|_| {
                Err(AuthenticationError::PersistenceError(
                    AuthenticationPersistenceError::TestVariant,
                ))
            });

        let result = under_test(
            mock_branch_counter_service,
            mock_github_authentication_service,
        )
        .authenticate_app_with_github(github_credentials)
        .await;

        assert_that(&matches!(
            result.err().unwrap(),
            ApplicationError::Domain(DomainError::Authentication(AuthenticationError::PersistenceError {..}))
        ))
        .is_true();
    }
}
