pub use github_token::*;

use crate::application::repository::RepositoryUrlDto;
use crate::ports::cli::commands::CliCommand;

pub mod commands;
mod github_token;
pub mod structopt;

pub trait ClientOptions {
    fn command(&self) -> CliCommand;

    fn repository_urls(&self) -> Option<&Vec<RepositoryUrlDto>>;

    fn github_auth_token(&self) -> Option<GitHubAuthenticationToken>;
}
