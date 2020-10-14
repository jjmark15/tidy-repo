use std::collections::HashMap;

use crate::application::{ApplicationError, BranchNameDto, RepositoryHostError, RepositoryUrlDto};
use crate::ports::repository_hosting::adapters::RepositoryHost;

pub struct ApplicationService<RepoHost: RepositoryHost> {
    repository_host: RepoHost,
}

impl<RepoHost> ApplicationService<RepoHost>
where
    RepoHost: RepositoryHost,
    <RepoHost as RepositoryHost>::Err: Into<RepositoryHostError>,
{
    pub async fn count_branches_in_repositories(
        &self,
        repository_urls: Vec<RepositoryUrlDto>,
    ) -> Result<HashMap<RepositoryUrlDto, u32>, ApplicationError> {
        let mut hash_map: HashMap<RepositoryUrlDto, u32> = HashMap::new();

        for url in repository_urls {
            let branches: Vec<BranchNameDto> =
                Self::list_branches(&self.repository_host, &url).await?;
            hash_map.insert(url.clone(), branches.len() as u32);
        }
        Ok(hash_map)
    }

    async fn list_branches(
        repository_host: &RepoHost,
        repository_url: &RepositoryUrlDto,
    ) -> Result<Vec<BranchNameDto>, RepositoryHostError> {
        match repository_host.list_branches(repository_url).await {
            Ok(branches) => Ok(branches),
            Err(e) => Err(e.into()),
        }
    }

    pub fn new(repository_host: RepoHost) -> Self {
        ApplicationService { repository_host }
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::eq;
    use spectral::prelude::*;

    use crate::application::BranchNameDto;
    use crate::ports::repository_hosting::adapters::MockRepositoryHost;

    use super::*;

    fn to_hash_map(list: Vec<(RepositoryUrlDto, u32)>) -> HashMap<RepositoryUrlDto, u32> {
        let mut hash_map = HashMap::new();
        list.iter().for_each(|(url, count)| {
            hash_map.insert(url.clone(), *count);
        });
        hash_map
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

    fn mock_repository_host() -> MockRepositoryHost {
        let repository_url_1 = RepositoryUrlDto::new("1".to_string());
        let repository_url_2 = RepositoryUrlDto::new("2".to_string());
        let mut mock_repository_host = MockRepositoryHost::default();
        prepare_mock_repository_host(
            &mut mock_repository_host,
            repository_url_1,
            to_branch_names(vec!["1"]),
        );
        prepare_mock_repository_host(
            &mut mock_repository_host,
            repository_url_2,
            to_branch_names(vec!["1", "2"]),
        );
        mock_repository_host
    }

    fn to_branch_names(branch_name_strings: Vec<&str>) -> Vec<BranchNameDto> {
        branch_name_strings
            .iter()
            .map(|s| BranchNameDto::new(s.to_string()))
            .collect()
    }

    fn to_urls(repository_url_strings: Vec<&str>) -> Vec<RepositoryUrlDto> {
        repository_url_strings
            .iter()
            .map(|s| RepositoryUrlDto::new(s.to_string()))
            .collect()
    }

    fn under_test(repository_host: MockRepositoryHost) -> ApplicationService<MockRepositoryHost> {
        ApplicationService::new(repository_host)
    }

    #[async_std::test]
    async fn counts_branches_in_list_of_repositories() {
        let mock_repository_host = mock_repository_host();

        assert_that(
            &under_test(mock_repository_host)
                .count_branches_in_repositories(to_urls(vec!["1", "2"]))
                .await
                .unwrap(),
        )
        .is_equal_to(&to_hash_map(vec![
            (RepositoryUrlDto::new("1".to_string()), 1u32),
            (RepositoryUrlDto::new("2".to_string()), 2u32),
        ]));
    }
}
