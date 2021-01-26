#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BranchName(String);

impl BranchName {
    pub fn new(value: String) -> Self {
        BranchName(value)
    }

    pub fn value(&self) -> &String {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use spectral::prelude::*;

    use super::*;

    #[test]
    fn returns_value() {
        assert_that(&BranchName("branch name".to_string()).value())
            .is_equal_to(&"branch name".to_string());
    }
}
