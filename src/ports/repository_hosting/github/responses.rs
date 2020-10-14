pub use branch::*;
pub use list_branches_response::*;

mod list_branches_response {
    use super::Branch;

    #[derive(Debug, serde::Serialize, serde::Deserialize)]
    #[serde(transparent)]
    pub struct ListBranchesResponseBody {
        branches: Vec<Branch>,
    }

    impl ListBranchesResponseBody {
        #[cfg(test)]
        pub fn new(branches: Vec<Branch>) -> Self {
            ListBranchesResponseBody { branches }
        }

        pub fn branches(&self) -> &Vec<Branch> {
            &self.branches
        }
    }
}

mod branch {
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct Branch {
        name: String,
    }

    impl Branch {
        #[cfg(test)]
        pub fn new(name: String) -> Self {
            Branch { name }
        }

        pub fn name(&self) -> &String {
            &self.name
        }
    }
}
