use crate::application::RepositoryUrlDto;
use crate::ports::cli::commands::CliCommand;

pub trait ClientOptions {
    fn command(&self) -> CliCommand;

    fn repository_urls(&self) -> Option<&Vec<RepositoryUrlDto>>;
}
