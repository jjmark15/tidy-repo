use structopt::StructOpt;

use crate::application::RepositoryUrlDto;
use crate::ports::cli::adapters::ClientOptions;

#[derive(StructOpt, Debug)]
#[structopt(name = "Tidy Repo")]
pub enum StructOptClientOptions {
    /// Get info relating to branches in a repository
    Branches {
        /// Repository URLs to process
        #[structopt(name = "REPOSITORY_URL")]
        repository_urls: Vec<RepositoryUrlDto>,
    },
}

impl ClientOptions for StructOptClientOptions {
    fn repository_urls(&self) -> &Vec<RepositoryUrlDto> {
        match self {
            StructOptClientOptions::Branches { repository_urls } => &repository_urls,
        }
    }
}

#[cfg(test)]
mod tests {
    use spectral::prelude::*;

    use super::*;

    fn under_test() -> StructOptClientOptions {
        StructOptClientOptions::Branches {
            repository_urls: vec![RepositoryUrlDto::new("url".to_string())],
        }
    }

    #[test]
    fn returns_list_of_repository_urls() {
        assert_that(&under_test().repository_urls())
            .is_equal_to(&vec![RepositoryUrlDto::new("url".to_string())])
    }
}
