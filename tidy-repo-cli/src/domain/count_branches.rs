use async_trait::async_trait;

use crate::domain::branch::Branch;
use crate::domain::error::DomainError;
use crate::domain::repository::RepositoryUrl;
use crate::domain::repository_host::RepositoryHostWrapper;

#[async_trait]
#[cfg_attr(test, mockall::automock)]
pub trait BranchCounterService {
    async fn count_branches(&mut self, repository_url: RepositoryUrl) -> Result<u32, DomainError>;
}

pub struct BranchCounterServiceImpl<RH>
where
    RH: RepositoryHostWrapper,
{
    repository_host: RH,
}

impl<RH> BranchCounterServiceImpl<RH>
where
    RH: RepositoryHostWrapper,
{
    pub fn new(repository_host: RH) -> Self {
        BranchCounterServiceImpl { repository_host }
    }

    async fn list_branches(
        &mut self,
        repository_url: &RepositoryUrl,
    ) -> Result<Vec<Branch>, DomainError> {
        self.repository_host
            .list_branches(&repository_url)
            .await
            .map_err(|err| err.into())
    }
}

#[async_trait]
impl<RH> BranchCounterService for BranchCounterServiceImpl<RH>
where
    RH: RepositoryHostWrapper + Send + Sync,
{
    async fn count_branches(&mut self, repository_url: RepositoryUrl) -> Result<u32, DomainError> {
        let branches = self.list_branches(&repository_url).await?;
        Ok(branches.len() as u32)
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::eq;
    use spectral::prelude::*;

    use crate::domain::error::RepositoryHostError;
    use crate::domain::repository_host::MockRepositoryHostWrapper;
    use crate::ports::repository_hosting::TestRepositoryHostError;

    use super::*;

    fn mock_repository_host_wrapper() -> MockRepositoryHostWrapper<String> {
        MockRepositoryHostWrapper::default()
    }

    fn prepare_mock_repository_host_wrapper(
        mock_repository_host_wrapper: &mut MockRepositoryHostWrapper<String>,
        url: RepositoryUrl,
        branches: Vec<Branch>,
    ) {
        mock_repository_host_wrapper
            .expect_list_branches()
            .with(eq(url))
            .returning(move |_| Ok(branches.clone()));
    }

    #[async_std::test]
    async fn counts_branches_in_a_repository_none() {
        let mut repository_host_wrapper = mock_repository_host_wrapper();
        prepare_mock_repository_host_wrapper(
            &mut repository_host_wrapper,
            RepositoryUrl::new("url".to_string()),
            vec![],
        );
        let mut under_test = BranchCounterServiceImpl::new(repository_host_wrapper);
        let repository_url = RepositoryUrl::new("url".to_string());

        assert_that(&under_test.count_branches(repository_url).await.unwrap()).is_equal_to(0);
    }

    #[async_std::test]
    async fn counts_branches_in_a_repository_single() {
        let mut repository_host_wrapper = mock_repository_host_wrapper();
        prepare_mock_repository_host_wrapper(
            &mut repository_host_wrapper,
            RepositoryUrl::new("url".to_string()),
            vec![Branch::new("1".to_string())],
        );
        let mut under_test = BranchCounterServiceImpl::new(repository_host_wrapper);
        let repository_url = RepositoryUrl::new("url".to_string());

        assert_that(&under_test.count_branches(repository_url).await.unwrap()).is_equal_to(1);
    }

    #[async_std::test]
    async fn counts_branches_in_a_repository_multiple() {
        let mut repository_host_wrapper = mock_repository_host_wrapper();
        prepare_mock_repository_host_wrapper(
            &mut repository_host_wrapper,
            RepositoryUrl::new("url".to_string()),
            vec![Branch::new("1".to_string()), Branch::new("2".to_string())],
        );
        let mut under_test = BranchCounterServiceImpl::new(repository_host_wrapper);
        let repository_url = RepositoryUrl::new("url".to_string());

        assert_that(&under_test.count_branches(repository_url).await.unwrap()).is_equal_to(2);
    }

    #[async_std::test]
    async fn errors_when_repository_host_errors() {
        let mut repository_host_wrapper = mock_repository_host_wrapper();
        repository_host_wrapper
            .expect_list_branches()
            .with(eq(RepositoryUrl::new("url".to_string())))
            .returning(move |_| {
                Err(RepositoryHostError::TestRepositoryHost(
                    TestRepositoryHostError,
                ))
            });
        let mut under_test = BranchCounterServiceImpl::new(repository_host_wrapper);
        let repository_url = RepositoryUrl::new("url".to_string());

        let result = under_test.count_branches(repository_url).await;

        assert_that(&matches!(result.err().unwrap(), DomainError::RepositoryHost {..})).is_true();
    }
}
