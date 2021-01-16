use crate::domain::authentication::GitHubAuthenticationToken;
use crate::ports::repository_hosting::github::GitHubAuthenticationToken as RepositoryHostingGitHubAuthenticationToken;

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
