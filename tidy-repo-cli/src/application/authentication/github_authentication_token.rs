use std::str::FromStr;

use crate::domain::authentication::GitHubAuthenticationToken;
use crate::ports::persistence::GitHubAuthenticationToken as PersistenceGitHubAuthenticationToken;
use crate::ports::repository_hosting::adapters::github::GitHubAuthenticationToken as RepositoryHostingGitHubAuthenticationToken;

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
