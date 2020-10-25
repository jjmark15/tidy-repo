use async_trait::async_trait;

use crate::domain::branch::Branch;
use crate::domain::error::{DomainError, RepositoryHostError};
use crate::domain::repository::RepositoryUrl;
use crate::domain::repository_host::RepositoryHostWrapper;
use crate::ports::repository_hosting::adapters::RepositoryHost;

#[async_trait]
#[cfg_attr(test, mockall::automock)]
pub trait BranchCounterService {
    async fn count_branches(&self, repository_url: RepositoryUrl) -> Result<u32, DomainError>;
}

pub struct BranchCounterServiceImpl<RH>
where
    RH: RepositoryHost,
    <RH as RepositoryHost>::Err: Into<RepositoryHostError>,
{
    repository_host: RepositoryHostWrapper<RH>,
}

impl<RH> BranchCounterServiceImpl<RH>
where
    RH: RepositoryHost,
    <RH as RepositoryHost>::Err: Into<RepositoryHostError>,
{
    pub fn new(repository_host: RepositoryHostWrapper<RH>) -> Self {
        BranchCounterServiceImpl { repository_host }
    }

    async fn list_branches(
        &self,
        repository_url: &RepositoryUrl,
    ) -> Result<Vec<Branch>, DomainError> {
        self.repository_host
            .list_branches(&repository_url)
            .await
            .map_err(|err| err.into())
    }
}

#[async_trait]
impl<RepoHost> BranchCounterService for BranchCounterServiceImpl<RepoHost>
where
    RepoHost: RepositoryHost + Send + Sync,
    <RepoHost as RepositoryHost>::Err: Into<RepositoryHostError>,
{
    async fn count_branches(&self, repository_url: RepositoryUrl) -> Result<u32, DomainError> {
        let branches = self.list_branches(&repository_url).await?;
        Ok(branches.len() as u32)
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::eq;
    use spectral::prelude::*;

    use crate::application::{BranchNameDto, RepositoryUrlDto};
    use crate::ports::repository_hosting::adapters::{MockRepositoryHost, TestRepositoryHostError};

    use super::*;

    fn mock_repository_host() -> MockRepositoryHost {
        MockRepositoryHost::default()
    }

    fn prepare_mock_repository_host(
        mock_repository_host: &mut MockRepositoryHost,
        url: RepositoryUrlDto,
        branches: Vec<BranchNameDto>,
    ) {
        mock_repository_host
            .expect_list_branches()
            .with(eq(url))
            .returning(move |_| Ok(branches.clone()));
    }

    #[async_std::test]
    async fn counts_branches_in_a_repository_none() {
        let mut mock_repository_host = mock_repository_host();
        prepare_mock_repository_host(
            &mut mock_repository_host,
            RepositoryUrlDto::new("url".to_string()),
            vec![],
        );
        let repository_host_wrapper = RepositoryHostWrapper::new(mock_repository_host);
        let under_test = BranchCounterServiceImpl::new(repository_host_wrapper);
        let repository_url = RepositoryUrl::new("url".to_string());

        assert_that(&under_test.count_branches(repository_url).await.unwrap()).is_equal_to(0);
    }

    #[async_std::test]
    async fn counts_branches_in_a_repository_single() {
        let mut mock_repository_host = mock_repository_host();
        prepare_mock_repository_host(
            &mut mock_repository_host,
            RepositoryUrlDto::new("url".to_string()),
            vec![BranchNameDto::new("1".to_string())],
        );
        let repository_host_wrapper = RepositoryHostWrapper::new(mock_repository_host);
        let under_test = BranchCounterServiceImpl::new(repository_host_wrapper);
        let repository_url = RepositoryUrl::new("url".to_string());

        assert_that(&under_test.count_branches(repository_url).await.unwrap()).is_equal_to(1);
    }

    #[async_std::test]
    async fn counts_branches_in_a_repository_multiple() {
        let mut mock_repository_host = mock_repository_host();
        prepare_mock_repository_host(
            &mut mock_repository_host,
            RepositoryUrlDto::new("url".to_string()),
            vec![
                BranchNameDto::new("1".to_string()),
                BranchNameDto::new("2".to_string()),
            ],
        );
        let repository_host_wrapper = RepositoryHostWrapper::new(mock_repository_host);
        let under_test = BranchCounterServiceImpl::new(repository_host_wrapper);
        let repository_url = RepositoryUrl::new("url".to_string());

        assert_that(&under_test.count_branches(repository_url).await.unwrap()).is_equal_to(2);
    }

    #[async_std::test]
    async fn errors_when_repository_host_errors() {
        let mut mock_repository_host = mock_repository_host();
        mock_repository_host
            .expect_list_branches()
            .with(eq(RepositoryUrlDto::new("url".to_string())))
            .returning(move |_| Err(TestRepositoryHostError));

        let repository_host_wrapper = RepositoryHostWrapper::new(mock_repository_host);
        let under_test = BranchCounterServiceImpl::new(repository_host_wrapper);
        let repository_url = RepositoryUrl::new("url".to_string());

        assert_that(&under_test.count_branches(repository_url).await).is_err();
    }
}
