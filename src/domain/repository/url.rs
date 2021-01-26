use crate::domain::value_object::ValueObject;

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct RepositoryUrl(String);

impl ValueObject<String> for RepositoryUrl {
    fn value(&self) -> &String {
        &self.0
    }
}

impl RepositoryUrl {
    pub fn new(value: String) -> Self {
        RepositoryUrl(value)
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
}
