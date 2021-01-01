use crate::application::repository::RepositoryUrlDto;
use crate::ports::cli::commands::CliCommand;
use crate::ports::cli::GitHubAuthenticationToken;

pub trait ClientOptions {
    fn command(&self) -> CliCommand;

    fn repository_urls(&self) -> Option<&Vec<RepositoryUrlDto>>;

    fn github_auth_token(&self) -> Option<GitHubAuthenticationToken>;
}
