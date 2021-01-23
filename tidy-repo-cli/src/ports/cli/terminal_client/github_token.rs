use std::str::FromStr;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct GitHubAuthenticationToken(String);

impl GitHubAuthenticationToken {
    pub fn new(token: String) -> Self {
        GitHubAuthenticationToken(token)
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}

impl FromStr for GitHubAuthenticationToken {
    type Err = GitHubAuthenticationTokenParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            Err(GitHubAuthenticationTokenParseError::Empty)
        } else {
            Ok(GitHubAuthenticationToken(s.to_string()))
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GitHubAuthenticationTokenParseError {
    #[error("GitHub authentication token must not be empty")]
    Empty,
}

#[cfg(test)]
mod tests {
    use spectral::prelude::*;

    use super::*;

    #[test]
    fn returns_string_value() {
        assert_that(&GitHubAuthenticationToken::new("token".to_string()).value())
            .is_equal_to("token");
    }

    #[test]
    fn parses_from_a_non_empty_string() {
        assert_that(&GitHubAuthenticationToken::from_str("token").unwrap())
            .is_equal_to(&GitHubAuthenticationToken::new("token".to_string()));
    }

    #[test]
    fn fails_to_parse_from_an_empty_string() {
        let result = GitHubAuthenticationToken::from_str("");
        assert_that(
            &matches!(result.err().unwrap(), GitHubAuthenticationTokenParseError::Empty {..}),
        )
        .is_true();
    }
}
