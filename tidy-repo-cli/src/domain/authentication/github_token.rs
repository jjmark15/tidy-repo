use std::convert::Infallible;
use std::str::FromStr;

use crate::ports::persistence::GitHubAuthenticationToken as PersistenceGitHubAuthenticationToken;
use crate::ports::repository_hosting::adapters::github::GitHubAuthenticationToken as RepositoryHostingGitHubAuthenticationToken;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct GitHubAuthenticationToken(String);

impl GitHubAuthenticationToken {
    pub fn new(value: String) -> Self {
        GitHubAuthenticationToken(value)
    }

    pub fn value(&self) -> String {
        self.0.clone()
    }
}

impl FromStr for GitHubAuthenticationToken {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(GitHubAuthenticationToken(s.to_string()))
    }
}

impl From<PersistenceGitHubAuthenticationToken> for GitHubAuthenticationToken {
    fn from(token: PersistenceGitHubAuthenticationToken) -> Self {
        GitHubAuthenticationToken::new(token.value())
    }
}

impl Into<PersistenceGitHubAuthenticationToken> for GitHubAuthenticationToken {
    fn into(self) -> PersistenceGitHubAuthenticationToken {
        PersistenceGitHubAuthenticationToken::from_str(self.value().as_str()).unwrap()
    }
}

impl From<RepositoryHostingGitHubAuthenticationToken> for GitHubAuthenticationToken {
    fn from(token: RepositoryHostingGitHubAuthenticationToken) -> Self {
        GitHubAuthenticationToken::new(token.value().to_string())
    }
}

impl From<GitHubAuthenticationToken> for RepositoryHostingGitHubAuthenticationToken {
    fn from(token: GitHubAuthenticationToken) -> Self {
        RepositoryHostingGitHubAuthenticationToken::new(token.value())
    }
}
