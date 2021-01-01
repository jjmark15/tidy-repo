pub use branch::*;
pub use provider::*;
pub use url::*;

mod branch;
mod provider;
mod url;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Repository {
    url: RepositoryUrl,
    branches: Vec<Branch>,
}

impl Repository {
    pub fn new(url: RepositoryUrl, branches: Vec<Branch>) -> Self {
        Repository { url, branches }
    }

    pub fn branches(&self) -> &Vec<Branch> {
        &self.branches
    }

    pub fn url(&self) -> &RepositoryUrl {
        &self.url
    }
}
