use async_trait::async_trait;

use crate::domain::branch::Branch;
use crate::domain::error::{DomainError, RepositoryHostError};
use crate::domain::repository::RepositoryUrl;
use crate::domain::repository_host::RepositoryHost;

#[async_trait]
#[cfg_attr(test, mockall::automock)]
pub trait BranchCounterService {
    async fn count_branches(&self, repository_url: RepositoryUrl) -> Result<u32, DomainError>;
}

pub struct BranchCounterServiceImpl<RepoHost>
where
    RepoHost: RepositoryHost,
{
    repository_host: RepoHost,
}

impl<RepoHost> BranchCounterServiceImpl<RepoHost>
where
    RepoHost: RepositoryHost + Send + Sync,
    <RepoHost as RepositoryHost>::Err: Into<RepositoryHostError>,
{
    pub fn new(repository_host: RepoHost) -> Self {
        BranchCounterServiceImpl { repository_host }
    }

    async fn list_branches(
        &self,
        repository_url: &RepositoryUrl,
    ) -> Result<Vec<Branch>, DomainError> {
        self.repository_host
            .list_branches(&repository_url)
            .await
            .map_err(|err| err.into().into())
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

    use crate::domain::repository_host::{MockRepositoryHost, TestRepositoryHostError};

    use super::*;

    fn mock_repository_host() -> MockRepositoryHost {
        MockRepositoryHost::default()
    }

    fn prepare_mock_repository_host(
        mock_repository_host: &mut MockRepositoryHost,
        url: RepositoryUrl,
        branches: Vec<Branch>,
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
            RepositoryUrl::new("url".to_string()),
            vec![],
        );
        let under_test = BranchCounterServiceImpl::new(mock_repository_host);
        let repository_url = RepositoryUrl::new("url".to_string());

        assert_that(&under_test.count_branches(repository_url).await.unwrap()).is_equal_to(0);
    }

    #[async_std::test]
    async fn counts_branches_in_a_repository_single() {
        let mut mock_repository_host = mock_repository_host();
        prepare_mock_repository_host(
            &mut mock_repository_host,
            RepositoryUrl::new("url".to_string()),
            vec![Branch::new("1".to_string())],
        );
        let under_test = BranchCounterServiceImpl::new(mock_repository_host);
        let repository_url = RepositoryUrl::new("url".to_string());

        assert_that(&under_test.count_branches(repository_url).await.unwrap()).is_equal_to(1);
    }

    #[async_std::test]
    async fn counts_branches_in_a_repository_multiple() {
        let mut mock_repository_host = mock_repository_host();
        prepare_mock_repository_host(
            &mut mock_repository_host,
            RepositoryUrl::new("url".to_string()),
            vec![Branch::new("1".to_string()), Branch::new("2".to_string())],
        );
        let under_test = BranchCounterServiceImpl::new(mock_repository_host);
        let repository_url = RepositoryUrl::new("url".to_string());

        assert_that(&under_test.count_branches(repository_url).await.unwrap()).is_equal_to(2);
    }

    #[async_std::test]
    async fn errors_when_repository_host_errors() {
        let mut mock_repository_host = mock_repository_host();
        mock_repository_host
            .expect_list_branches()
            .with(eq(RepositoryUrl::new("url".to_string())))
            .returning(move |_| Err(TestRepositoryHostError));

        let under_test = BranchCounterServiceImpl::new(mock_repository_host);
        let repository_url = RepositoryUrl::new("url".to_string());

        assert_that(&under_test.count_branches(repository_url).await).is_err();
    }
}
