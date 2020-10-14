#[derive(Debug, Eq, PartialEq)]
pub struct GitHubRepository {
    owner: String,
    name: String,
}

impl GitHubRepository {
    pub fn new(owner: String, name: String) -> Self {
        GitHubRepository { owner, name }
    }

    pub fn owner(&self) -> &String {
        &self.owner
    }

    pub fn name(&self) -> &String {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spectral::prelude::*;

    #[test]
    fn returns_owner() {
        let under_test = GitHubRepository::new("owner".to_string(), "name".to_string());
        assert_that(&under_test.owner()).is_equal_to(&"owner".to_string());
    }

    #[test]
    fn returns_name() {
        let under_test = GitHubRepository::new("owner".to_string(), "name".to_string());
        assert_that(&under_test.name()).is_equal_to(&"name".to_string());
    }
}
