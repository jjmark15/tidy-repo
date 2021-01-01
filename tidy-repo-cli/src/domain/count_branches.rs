use crate::domain::repository::Repository;

#[cfg_attr(test, mockall::automock)]
pub trait BranchCounterService {
    fn count_branches_in_repositories(
        &self,
        repositories: Vec<Repository>,
    ) -> Vec<(Repository, u32)>;
}

#[derive(Default)]
pub struct BranchCounterServiceImpl;

impl BranchCounterServiceImpl {
    pub fn new() -> Self {
        BranchCounterServiceImpl
    }
}

impl BranchCounterService for BranchCounterServiceImpl {
    fn count_branches_in_repositories(
        &self,
        repositories: Vec<Repository>,
    ) -> Vec<(Repository, u32)> {
        repositories
            .iter()
            .map(|repo| (repo.clone(), repo.branches().len() as u32))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use spectral::prelude::*;

    use crate::domain::repository::Branch;
    use crate::domain::repository::RepositoryUrl;

    use super::*;

    #[test]
    fn counts_branches_in_a_repository_none() {
        let under_test = BranchCounterServiceImpl::new();
        let repository = Repository::new(RepositoryUrl::new("url".to_string()), vec![]);

        assert_that(&under_test.count_branches_in_repositories(vec![repository.clone()]))
            .is_equal_to(vec![(repository, 0)]);
    }

    #[test]
    fn counts_branches_in_a_repository_single() {
        let under_test = BranchCounterServiceImpl::new();
        let repository = Repository::new(
            RepositoryUrl::new("url".to_string()),
            vec![Branch::new("1".to_string())],
        );

        assert_that(&under_test.count_branches_in_repositories(vec![repository.clone()]))
            .is_equal_to(vec![(repository, 1)]);
    }

    #[test]
    fn counts_branches_in_a_repository_multiple() {
        let under_test = BranchCounterServiceImpl::new();
        let repository = Repository::new(
            RepositoryUrl::new("url".to_string()),
            vec![Branch::new("1".to_string()), Branch::new("2".to_string())],
        );

        assert_that(&under_test.count_branches_in_repositories(vec![repository.clone()]))
            .is_equal_to(vec![(repository, 2)]);
    }
}
