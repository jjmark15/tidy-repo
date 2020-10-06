use crate::application::RepositoryUrlDto;

pub trait RepositoryClient {
    fn count_branches(&self, repository_url: &RepositoryUrlDto) -> u32;
}

#[cfg(test)]
mockall::mock! {
    pub RepositoryClient {}

    pub trait RepositoryClient {
        fn count_branches(&self, repository_url: &RepositoryUrlDto) -> u32;
    }
}
