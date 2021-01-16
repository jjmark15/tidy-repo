use std::fmt::Formatter;

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct RepositoryUrl(String);

impl RepositoryUrl {
    pub fn new(value: String) -> Self {
        RepositoryUrl(value)
    }

    pub fn value(&self) -> &String {
        &self.0
    }
}

impl core::fmt::Display for RepositoryUrl {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use spectral::prelude::*;

    use super::*;

    fn under_test() -> RepositoryUrl {
        RepositoryUrl::new("url".to_string())
    }

    #[test]
    fn returns_string_value() {
        assert_that(&under_test().value()).is_equal_to(&"url".to_string());
    }

    #[test]
    fn implements_display() {
        assert_that(&under_test().to_string()).is_equal_to(&"url".to_string());
    }
}
