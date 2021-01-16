use structopt::StructOpt;

use crate::application::repository::RepositoryUrlDto;
use crate::ports::initiation::terminal_client::commands::CliCommand;
use crate::ports::initiation::terminal_client::github_token::GitHubAuthenticationToken;

pub trait ClientOptions {
    fn command(&self) -> CliCommand;

    fn repository_urls(&self) -> Option<&Vec<RepositoryUrlDto>>;

    fn github_auth_token(&self) -> Option<GitHubAuthenticationToken>;
}

#[derive(StructOpt, Debug)]
#[structopt(name = "Tidy Repo")]
pub enum StructOptClientOptions {
    /// Authenticate with repository hosting services
    Authenticate(AuthenticateCommand),
    /// Get info relating to branches in a repository
    Branches {
        /// Repository URLs to process
        #[structopt(name = "REPOSITORY_URL")]
        repository_urls: Vec<RepositoryUrlDto>,
    },
}

#[derive(StructOpt, Debug)]
pub enum AuthenticateCommand {
    /// Authenticate with GitHub
    #[structopt(name = "github")]
    GitHub {
        /// Personal access token
        #[structopt(name = "token", long, short)]
        token: GitHubAuthenticationToken,
    },
}

impl ClientOptions for StructOptClientOptions {
    fn command(&self) -> CliCommand {
        match self {
            StructOptClientOptions::Authenticate(AuthenticateCommand::GitHub { .. }) => {
                CliCommand::AuthenticateGitHub
            }
            StructOptClientOptions::Branches { .. } => CliCommand::Branches,
        }
    }

    fn repository_urls(&self) -> Option<&Vec<RepositoryUrlDto>> {
        match self {
            StructOptClientOptions::Branches { repository_urls } => Some(&repository_urls),
            _ => None,
        }
    }

    fn github_auth_token(&self) -> Option<GitHubAuthenticationToken> {
        match self {
            StructOptClientOptions::Authenticate(AuthenticateCommand::GitHub { token }) => {
                Some(token.clone())
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use spectral::prelude::*;

    use super::*;

    fn branches_options() -> StructOptClientOptions {
        StructOptClientOptions::Branches {
            repository_urls: vec![RepositoryUrlDto::new("url".to_string())],
        }
    }

    fn authenticate_github_options() -> StructOptClientOptions {
        StructOptClientOptions::Authenticate(AuthenticateCommand::GitHub {
            token: GitHubAuthenticationToken::new("token".to_string()),
        })
    }

    #[test]
    fn returns_list_of_repository_urls_when_counting_branches() {
        assert_that(&branches_options().repository_urls().unwrap())
            .is_equal_to(&vec![RepositoryUrlDto::new("url".to_string())])
    }

    #[test]
    fn returns_none_when_not_counting_branches() {
        assert_that(&authenticate_github_options().repository_urls()).is_none();
    }

    #[test]
    fn returns_github_token_when_authenticating_with_github() {
        assert_that(&authenticate_github_options().github_auth_token().unwrap())
            .is_equal_to(&GitHubAuthenticationToken::new("token".to_string()))
    }

    #[test]
    fn returns_none_when_not_authenticating_with_github() {
        assert_that(&branches_options().github_auth_token()).is_none();
    }
}
