use std::convert::{TryFrom, TryInto};

use crate::application::RepositoryUrlDto;
use crate::ports::repository_client::github::repository::GitHubRepository;
use regex::Regex;

#[cfg_attr(test, mockall::automock)]
pub trait GitHubRepositoryUrlParser {
    fn parse(&self, url: RepositoryUrlDto) -> Result<GitHubRepository, RepositoryUrlParseError>;
}

#[derive(Debug)]
pub struct GitHubRepositoryUrlParserImpl;

impl GitHubRepositoryUrlParserImpl {
    pub fn new() -> Self {
        GitHubRepositoryUrlParserImpl
    }
}

impl GitHubRepositoryUrlParser for GitHubRepositoryUrlParserImpl {
    fn parse(&self, url: RepositoryUrlDto) -> Result<GitHubRepository, RepositoryUrlParseError> {
        url.try_into()
    }
}

impl TryFrom<RepositoryUrlDto> for GitHubRepository {
    type Error = RepositoryUrlParseError;

    fn try_from(url: RepositoryUrlDto) -> Result<Self, Self::Error> {
        let re = Regex::new(r"(?:https?://)?github\.com/(?P<owner>\S+)/(?P<name>\S+)").unwrap();

        match re.captures(url.value()) {
            Some(captures) => match captures.name("owner") {
                Some(owner_match) => match captures.name("name") {
                    Some(name_match) => Ok(GitHubRepository::new(
                        owner_match.as_str().to_string(),
                        name_match.as_str().to_string(),
                    )),
                    _ => Err(RepositoryUrlParseError(url.value().clone())),
                },
                _ => Err(RepositoryUrlParseError(url.value().clone())),
            },
            _ => Err(RepositoryUrlParseError(url.value().clone())),
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("failed to parse repository from {0}")]
pub struct RepositoryUrlParseError(String);

#[cfg(test)]
mod tests {
    use spectral::prelude::*;

    use super::*;

    #[test]
    fn parses_github_repository_url_with_tls() {
        let url = RepositoryUrlDto::new("https://github.com/owner/repo".to_string());
        let under_test = GitHubRepositoryUrlParserImpl::new();
        assert_that(&under_test.parse(url).unwrap()).is_equal_to(GitHubRepository::new(
            "owner".to_string(),
            "repo".to_string(),
        ));
    }

    #[test]
    fn parses_github_repository_url_without_tls() {
        let url = RepositoryUrlDto::new("http://github.com/owner/repo".to_string());
        let under_test = GitHubRepositoryUrlParserImpl::new();
        assert_that(&under_test.parse(url).unwrap()).is_equal_to(GitHubRepository::new(
            "owner".to_string(),
            "repo".to_string(),
        ));
    }

    #[test]
    fn parses_github_repository_url_without_scheme() {
        let url = RepositoryUrlDto::new("github.com/owner/repo".to_string());
        let under_test = GitHubRepositoryUrlParserImpl::new();
        assert_that(&under_test.parse(url).unwrap()).is_equal_to(GitHubRepository::new(
            "owner".to_string(),
            "repo".to_string(),
        ));
    }

    #[test]
    fn fails_to_parse_github_repository_url_missing_owner() {
        let url = RepositoryUrlDto::new("https://github.com//repo".to_string());
        let under_test = GitHubRepositoryUrlParserImpl::new();
        assert_that(&under_test.parse(url)).is_err();
    }

    #[test]
    fn fails_to_parse_github_repository_url_missing_repo_name() {
        let url = RepositoryUrlDto::new("https://github.com/owner".to_string());
        let under_test = GitHubRepositoryUrlParserImpl::new();
        assert_that(&under_test.parse(url)).is_err();
    }
}
