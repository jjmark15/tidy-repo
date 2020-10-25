use crate::application::RepositoryUrlDto;
use crate::domain::branch::Branch;
use crate::domain::error::RepositoryHostError;
use crate::domain::repository::RepositoryUrl;
use crate::domain::value_object::ValueObject;
use crate::ports::repository_hosting::adapters::RepositoryHost;

pub struct RepositoryHostWrapper<RH>
where
    RH: RepositoryHost,
    <RH as RepositoryHost>::Err: Into<RepositoryHostError>,
{
    repository_host: RH,
}

impl<RH> RepositoryHostWrapper<RH>
where
    RH: RepositoryHost,
    <RH as RepositoryHost>::Err: Into<RepositoryHostError>,
{
    pub fn new(repository_host: RH) -> Self {
        RepositoryHostWrapper { repository_host }
    }

    pub async fn list_branches(
        &self,
        repository_url: &RepositoryUrl,
    ) -> Result<Vec<Branch>, RepositoryHostError> {
        let repo_url_dto = RepositoryUrlDto::new(repository_url.value());
        self.repository_host
            .list_branches(&repo_url_dto)
            .await
            .map(|res| {
                res.iter()
                    .map(|branch_name| Branch::new(branch_name.value().clone()))
                    .collect()
            })
            .map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::eq;
    use spectral::prelude::*;

    use crate::application::BranchNameDto;
    use crate::ports::repository_hosting::adapters::{MockRepositoryHost, TestRepositoryHostError};

    use super::*;

    fn under_test(
        repository_host: MockRepositoryHost,
    ) -> RepositoryHostWrapper<MockRepositoryHost> {
        RepositoryHostWrapper::new(repository_host)
    }

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
    async fn lists_branches_in_a_repository_none() {
        let mut mock_repository_host = mock_repository_host();
        prepare_mock_repository_host(
            &mut mock_repository_host,
            RepositoryUrlDto::new("url".to_string()),
            vec![],
        );
        let repository_url = RepositoryUrl::new("url".to_string());

        assert_that(
            &under_test(mock_repository_host)
                .list_branches(&repository_url)
                .await
                .unwrap(),
        )
        .is_equal_to(vec![]);
    }

    #[async_std::test]
    async fn lists_branches_in_a_repository_single() {
        let mut mock_repository_host = mock_repository_host();
        prepare_mock_repository_host(
            &mut mock_repository_host,
            RepositoryUrlDto::new("url".to_string()),
            vec![BranchNameDto::new("1".to_string())],
        );
        let repository_url = RepositoryUrl::new("url".to_string());

        assert_that(
            &under_test(mock_repository_host)
                .list_branches(&repository_url)
                .await
                .unwrap(),
        )
        .is_equal_to(vec![Branch::new("1".to_string())]);
    }

    #[async_std::test]
    async fn lists_branches_in_a_repository_multiple() {
        let mut mock_repository_host = mock_repository_host();
        prepare_mock_repository_host(
            &mut mock_repository_host,
            RepositoryUrlDto::new("url".to_string()),
            vec![
                BranchNameDto::new("1".to_string()),
                BranchNameDto::new("2".to_string()),
            ],
        );
        let repository_url = RepositoryUrl::new("url".to_string());

        assert_that(
            &under_test(mock_repository_host)
                .list_branches(&repository_url)
                .await
                .unwrap(),
        )
        .is_equal_to(vec![
            Branch::new("1".to_string()),
            Branch::new("2".to_string()),
        ]);
    }

    #[async_std::test]
    async fn errors_when_dependency_errors() {
        let mut mock_repository_host = mock_repository_host();
        mock_repository_host
            .expect_list_branches()
            .with(eq(RepositoryUrlDto::new("url".to_string())))
            .returning(move |_| Err(TestRepositoryHostError));
        let repository_url = RepositoryUrl::new("url".to_string());

        assert_that(
            &under_test(mock_repository_host)
                .list_branches(&repository_url)
                .await,
        )
        .is_err();
    }
}
