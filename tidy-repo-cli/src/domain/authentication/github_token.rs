use std::convert::Infallible;
use std::str::FromStr;

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
