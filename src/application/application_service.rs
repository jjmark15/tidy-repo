use std::collections::HashMap;

use crate::application::{
    ApplicationError, BranchNameDto, RepositoryClientError, RepositoryUrlDto,
};
use crate::ports::repository_client::adapters::RepositoryClient;

pub struct ApplicationService<RepoClient: RepositoryClient> {
    repository_client: RepoClient,
}

impl<RepoClient> ApplicationService<RepoClient>
where
    RepoClient: RepositoryClient,
    <RepoClient as RepositoryClient>::Err: Into<RepositoryClientError>,
{
    pub async fn count_branches_in_repositories(
        &self,
        repository_urls: Vec<RepositoryUrlDto>,
    ) -> Result<HashMap<RepositoryUrlDto, u32>, ApplicationError> {
        let mut hash_map: HashMap<RepositoryUrlDto, u32> = HashMap::new();

        for url in repository_urls {
            let branches: Vec<BranchNameDto> =
                Self::list_branches(&self.repository_client, &url).await?;
            hash_map.insert(url.clone(), branches.len() as u32);
        }
        Ok(hash_map)
    }

    async fn list_branches(
        repository_client: &RepoClient,
        repository_url: &RepositoryUrlDto,
    ) -> Result<Vec<BranchNameDto>, RepositoryClientError> {
        match repository_client.list_branches(repository_url).await {
            Ok(branches) => Ok(branches),
            Err(e) => Err(e.into()),
        }
    }

    pub fn new(repository_client: RepoClient) -> Self {
        ApplicationService { repository_client }
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::eq;
    use spectral::prelude::*;

    use crate::application::BranchNameDto;
    use crate::ports::repository_client::adapters::MockRepositoryClient;

    use super::*;

    fn to_hash_map(list: Vec<(RepositoryUrlDto, u32)>) -> HashMap<RepositoryUrlDto, u32> {
        let mut hash_map = HashMap::new();
        list.iter().for_each(|(url, count)| {
            hash_map.insert(url.clone(), *count);
        });
        hash_map
    }

    fn prepare_mock_repository_client(
        mock_repository_client: &mut MockRepositoryClient,
        url: RepositoryUrlDto,
        branches: Vec<BranchNameDto>,
    ) {
        mock_repository_client
            .expect_list_branches()
            .with(eq(url))
            .returning(move |_| Ok(branches.clone()));
    }

    fn mock_repository_client() -> MockRepositoryClient {
        let repository_url_1 = RepositoryUrlDto::new("1".to_string());
        let repository_url_2 = RepositoryUrlDto::new("2".to_string());
        let mut mock_repository_client = MockRepositoryClient::default();
        prepare_mock_repository_client(
            &mut mock_repository_client,
            repository_url_1,
            to_branch_names(vec!["1"]),
        );
        prepare_mock_repository_client(
            &mut mock_repository_client,
            repository_url_2,
            to_branch_names(vec!["1", "2"]),
        );
        mock_repository_client
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

    fn under_test(
        repository_client: MockRepositoryClient,
    ) -> ApplicationService<MockRepositoryClient> {
        ApplicationService::new(repository_client)
    }

    #[async_std::test]
    async fn counts_branches_in_list_of_repositories() {
        let mock_repository_client = mock_repository_client();

        assert_that(
            &under_test(mock_repository_client)
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
