use std::convert::Infallible;
use std::str::FromStr;

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
}
