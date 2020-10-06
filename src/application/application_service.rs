use std::collections::HashMap;

use crate::adapters::repository_client::RepositoryClient;
use crate::application::RepositoryUrlDto;

pub struct ApplicationService<RepoClient: RepositoryClient> {
    repository_client: RepoClient,
}

impl<RepoClient: RepositoryClient> ApplicationService<RepoClient> {
    pub fn count_branches_in_repositories(
        &self,
        repository_urls: Vec<RepositoryUrlDto>,
    ) -> HashMap<RepositoryUrlDto, u32> {
        let mut hash_map: HashMap<RepositoryUrlDto, u32> = HashMap::new();
        repository_urls.iter().for_each(|url| {
            let count = self.repository_client.count_branches(url);
            hash_map.insert(url.clone(), count);
        });
        hash_map
    }

    pub fn new(repository_client: RepoClient) -> Self {
        ApplicationService { repository_client }
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::eq;
    use spectral::prelude::*;

    use crate::adapters::repository_client::MockRepositoryClient;

    use super::*;

    fn from_list(list: Vec<(RepositoryUrlDto, u32)>) -> HashMap<RepositoryUrlDto, u32> {
        let mut hash_map = HashMap::new();
        list.iter().for_each(|(url, count)| {
            hash_map.insert(url.clone(), *count);
        });
        hash_map
    }

    fn mock_repository_client(counts: Vec<(RepositoryUrlDto, u32)>) -> MockRepositoryClient {
        let mut mock_repository_client = MockRepositoryClient::default();
        counts.iter().for_each(|(url, count)| {
            mock_repository_client
                .expect_count_branches()
                .with(eq(url.clone()))
                .return_const(*count);
        });
        mock_repository_client
    }

    #[test]
    fn counts_branches_in_list_of_repositories() {
        let repository_url_1 = RepositoryUrlDto::new("1".to_string());
        let repository_url_2 = RepositoryUrlDto::new("2".to_string());
        let mock_repository_client = mock_repository_client(vec![
            (repository_url_1.clone(), 1),
            (repository_url_2.clone(), 2),
        ]);
        let under_test = ApplicationService::<MockRepositoryClient>::new(mock_repository_client);

        let expected = from_list(vec![
            (RepositoryUrlDto::new("1".to_string()), 1u32),
            (RepositoryUrlDto::new("2".to_string()), 2u32),
        ]);
        assert_that(
            &under_test.count_branches_in_repositories(vec![repository_url_1, repository_url_2]),
        )
        .is_equal_to(&expected);
    }
}
