use std::convert::{TryFrom, TryInto};

use regex::Regex;

use crate::application::RepositoryUrlDto;
use crate::ports::repository_hosting::adapters::github::repository::GitHubRepository;

#[cfg_attr(test, mockall::automock)]
pub trait GitHubRepositoryUrlParser {
    fn parse(&self, url: RepositoryUrlDto) -> Result<GitHubRepository, RepositoryUrlParseError>;
}

#[derive(Debug, Default)]
pub struct GitHubRepositoryUrlParserImpl;

impl GitHubRepositoryUrlParserImpl {
    pub fn new() -> Self {
        GitHubRepositoryUrlParserImpl::default()
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
        let re = Regex::new(r"^(?:https?://)?github\.com/(?P<owner>\S+)/(?P<name>\S+)$").unwrap();

        if let Some(captures) = re.captures(url.value()) {
            if let Some(owner) = captures.name("owner") {
                if let Some(name) = captures.name("name") {
                    return Ok(GitHubRepository::new(
                        owner.as_str().to_string(),
                        name.as_str().to_string(),
                    ));
                }
            }
        }

        Err(RepositoryUrlParseError(url.value().clone()))
    }
}

#[derive(Debug, thiserror::Error)]
#[error("failed to parse repository from '{0}'")]
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
        let result = under_test.parse(url);
        assert_that(&matches!(result.err().unwrap(), RepositoryUrlParseError {..})).is_true();
    }

    #[test]
    fn fails_to_parse_github_repository_url_missing_repo_name() {
        let url = RepositoryUrlDto::new("https://github.com/owner".to_string());
        let under_test = GitHubRepositoryUrlParserImpl::new();
        let result = under_test.parse(url);
        assert_that(&matches!(result.err().unwrap(), RepositoryUrlParseError {..})).is_true();
    }

    #[test]
    fn fails_to_parse_github_repository_url_with_non_github_base_url() {
        let url = RepositoryUrlDto::new("https://not-github.com/owner/repo".to_string());
        let under_test = GitHubRepositoryUrlParserImpl::new();
        let result = under_test.parse(url);
        assert_that(&matches!(result.err().unwrap(), RepositoryUrlParseError {..})).is_true();
    }
}
