use std::convert::Infallible;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Credentials {
    github_token: GitHubAuthenticationToken,
}

impl Credentials {
    pub fn new(github_token: GitHubAuthenticationToken) -> Self {
        Credentials { github_token }
    }

    pub fn github_token(&self) -> &GitHubAuthenticationToken {
        &self.github_token
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
#[serde(transparent)]
pub struct GitHubAuthenticationToken(String);

impl GitHubAuthenticationToken {
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
