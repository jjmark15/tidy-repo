use std::convert::Infallible;
use std::fmt::Display;
use std::str::FromStr;

use serde::export::Formatter;

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct RepositoryUrlDto(String);

impl RepositoryUrlDto {
    pub fn value(&self) -> &String {
        &self.0
    }

    pub fn new(value: String) -> Self {
        RepositoryUrlDto(value)
    }
}

impl FromStr for RepositoryUrlDto {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(RepositoryUrlDto::new(s.to_string()))
    }
}

impl Display for RepositoryUrlDto {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use spectral::prelude::*;

    use super::*;

    fn under_test() -> RepositoryUrlDto {
        RepositoryUrlDto::new("url".to_string())
    }

    #[test]
    fn returns_value() {
        assert_that(&under_test().value()).is_equal_to(&"url".to_string());
    }

    #[test]
    fn implements_from_string_infallibly() {
        assert_that(&RepositoryUrlDto::from_str(&"url".to_string()).unwrap())
            .is_equal_to(&RepositoryUrlDto::new("url".to_string()));
    }

    #[test]
    fn to_string_equals_contained_string_url() {
        let under_test = RepositoryUrlDto::new("https://github.com/owner/repo".to_string());
        assert_that(&under_test.to_string())
            .is_equal_to(&"https://github.com/owner/repo".to_string())
    }
}
