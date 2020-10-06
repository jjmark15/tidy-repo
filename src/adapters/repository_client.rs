use crate::application::RepositoryUrlDto;

#[cfg_attr(test, mockall::automock)]
pub trait RepositoryClient {
    fn count_branches(&self, repository_url: &RepositoryUrlDto) -> u32;
}
