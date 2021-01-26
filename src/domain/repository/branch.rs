use crate::domain::value_object::ValueObject;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Branch {
    name: String,
}

impl Branch {
    pub fn new(name: String) -> Self {
        Branch { name }
    }
}

impl ValueObject<String> for Branch {
    fn value(&self) -> &String {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use spectral::prelude::*;

    use super::*;

    fn under_test() -> Branch {
        Branch::new("branch".to_string())
    }

    #[test]
    fn returns_string_value() {
        assert_that(&under_test().value()).is_equal_to(&"branch".to_string());
    }
}
