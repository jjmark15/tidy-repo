use crate::adapters::cli::ClientOptions;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "Tidy Repo")]
pub struct StructOptClientOptions {
    /// Repository URLs to process
    #[structopt(name = "REPOSITORY_URL")]
    repository_urls: Vec<String>,
}

impl ClientOptions for StructOptClientOptions {
    fn repository_urls(&self) -> &Vec<String> {
        &self.repository_urls
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spectral::prelude::*;

    fn under_test() -> StructOptClientOptions {
        StructOptClientOptions {
            repository_urls: vec!["url".to_string()],
        }
    }

    #[test]
    fn returns_list_of_repository_urls() {
        assert_that(&under_test().repository_urls()).is_equal_to(&vec!["url".to_string()])
    }
}
