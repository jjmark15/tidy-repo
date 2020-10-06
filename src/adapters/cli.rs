use crate::application::RepositoryUrlDto;

pub trait ClientOptions {
    fn repository_urls(&self) -> &Vec<RepositoryUrlDto>;
}
