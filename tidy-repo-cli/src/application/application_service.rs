use std::collections::HashMap;

use crate::application::{ApplicationError, RepositoryUrlDto};
use crate::domain::count_branches::BranchCounterService;

pub struct ApplicationService<BranchCounter: BranchCounterService> {
    branch_counter_service: BranchCounter,
}

impl<BranchCounter> ApplicationService<BranchCounter>
where
    BranchCounter: BranchCounterService,
{
    pub async fn count_branches_in_repositories(
        &self,
        repository_urls: Vec<RepositoryUrlDto>,
    ) -> Result<HashMap<RepositoryUrlDto, u32>, ApplicationError> {
        let mut hash_map: HashMap<RepositoryUrlDto, u32> = HashMap::new();

        for url in repository_urls {
            hash_map.insert(
                url.clone(),
                self.branch_counter_service
                    .count_branches(url.into())
                    .await?,
            );
        }
        Ok(hash_map)
    }

    pub fn new(branch_counter_service: BranchCounter) -> Self {
        ApplicationService {
            branch_counter_service,
        }
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::eq;
    use spectral::prelude::*;

    use crate::domain::count_branches::MockBranchCounterService;
    use crate::domain::repository::RepositoryUrl;

    use super::*;

    fn under_test(
        branch_counter_service: MockBranchCounterService,
    ) -> ApplicationService<MockBranchCounterService> {
        ApplicationService::new(branch_counter_service)
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

        assert_that(
            &under_test(mock_branch_counter_service)
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
