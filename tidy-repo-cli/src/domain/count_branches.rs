use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::iter::FromIterator;

use async_trait::async_trait;
use futures::future::try_join_all;

use crate::domain::branch::Branch;
use crate::domain::error::DomainError;
use crate::domain::repository::RepositoryUrl;
use crate::domain::repository_host::RepositoryHostWrapper;

#[async_trait]
#[cfg_attr(test, mockall::automock)]
pub trait BranchCounterService {
    async fn count_branches_in_repositories(
        &self,
        repository_urls: Vec<RepositoryUrl>,
    ) -> Result<HashMap<RepositoryUrl, u32>, DomainError>;
}

pub struct BranchCounterServiceImpl<RH>
where
    RH: RepositoryHostWrapper,
{
    repository_host: RH,
}

impl<RH> BranchCounterServiceImpl<RH>
where
    RH: RepositoryHostWrapper + Send + Sync,
{
    pub fn new(repository_host: RH) -> Self {
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

    async fn count_branches_in_repository(
        &self,
        repository_url: RepositoryUrl,
    ) -> Result<u32, DomainError> {
        let branches = self.list_branches(&repository_url).await?;
        Ok(branches.len() as u32)
    }

    async fn join_url_with_count(
        &self,
        url: RepositoryUrl,
    ) -> Result<(RepositoryUrl, u32), DomainError> {
        let count = self.count_branches_in_repository(url.clone()).await?;
        Ok((url, count))
    }
}

#[async_trait]
impl<RH> BranchCounterService for BranchCounterServiceImpl<RH>
where
    RH: RepositoryHostWrapper + Send + Sync,
{
    async fn count_branches_in_repositories(
        &self,
        repository_urls: Vec<RepositoryUrl>,
    ) -> Result<HashMap<RepositoryUrl, u32, RandomState>, DomainError> {
        Ok(HashMap::from_iter(
            try_join_all(
                repository_urls
                    .iter()
                    .map(|url| self.join_url_with_count(url.clone())),
            )
            .await?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::eq;
    use spectral::prelude::*;

    use crate::domain::repository_host::{MockRepositoryHostWrapper, RepositoryHostError};
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
        let under_test = BranchCounterServiceImpl::new(repository_host_wrapper);
        let repository_url = RepositoryUrl::new("url".to_string());

        assert_that(
            &under_test
                .count_branches_in_repository(repository_url)
                .await
                .unwrap(),
        )
        .is_equal_to(0);
    }

    #[async_std::test]
    async fn counts_branches_in_a_repository_single() {
        let mut repository_host_wrapper = mock_repository_host_wrapper();
        prepare_mock_repository_host_wrapper(
            &mut repository_host_wrapper,
            RepositoryUrl::new("url".to_string()),
            vec![Branch::new("1".to_string())],
        );
        let under_test = BranchCounterServiceImpl::new(repository_host_wrapper);
        let repository_url = RepositoryUrl::new("url".to_string());

        assert_that(
            &under_test
                .count_branches_in_repository(repository_url)
                .await
                .unwrap(),
        )
        .is_equal_to(1);
    }

    #[async_std::test]
    async fn counts_branches_in_a_repository_multiple() {
        let mut repository_host_wrapper = mock_repository_host_wrapper();
        prepare_mock_repository_host_wrapper(
            &mut repository_host_wrapper,
            RepositoryUrl::new("url".to_string()),
            vec![Branch::new("1".to_string()), Branch::new("2".to_string())],
        );
        let under_test = BranchCounterServiceImpl::new(repository_host_wrapper);
        let repository_url = RepositoryUrl::new("url".to_string());

        assert_that(
            &under_test
                .count_branches_in_repository(repository_url)
                .await
                .unwrap(),
        )
        .is_equal_to(2);
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
        let under_test = BranchCounterServiceImpl::new(repository_host_wrapper);
        let repository_url = RepositoryUrl::new("url".to_string());

        let result = under_test
            .count_branches_in_repository(repository_url)
            .await;

        assert_that(&matches!(result.err().unwrap(), DomainError::RepositoryHost {..})).is_true();
    }
}
