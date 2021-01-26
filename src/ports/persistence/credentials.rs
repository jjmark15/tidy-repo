use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct Credentials {
    github_token: String,
}

impl Credentials {
    pub fn new(github_token: String) -> Self {
        Credentials { github_token }
    }

    pub fn github_token(&self) -> &str {
        self.github_token.as_str()
    }
}
