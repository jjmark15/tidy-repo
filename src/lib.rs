use crate::adapters::cli::ClientOptions;
use crate::adapters::repository_client::RepositoryClient;
use crate::application::{ApplicationService, RepositoryClientError};
use crate::cli_results::CountBranchesResult;

mod adapters;
pub mod application;
mod cli_results;
mod domain;
pub mod ports;
pub mod utils;

pub struct TidyRepoClient<CO, RepoClient>
where
    CO: ClientOptions,
    RepoClient: RepositoryClient,
    <RepoClient as RepositoryClient>::Err: Into<RepositoryClientError>,
{
    client_options: CO,
    application_service: ApplicationService<RepoClient>,
}

impl<CO, RepoClient> TidyRepoClient<CO, RepoClient>
where
    CO: ClientOptions,
    RepoClient: RepositoryClient,
    <RepoClient as RepositoryClient>::Err: Into<RepositoryClientError>,
{
    pub fn new(client_options: CO, application_service: ApplicationService<RepoClient>) -> Self {
        TidyRepoClient {
            client_options,
            application_service,
        }
    }

    async fn count_branches_in_repositories(&self) {
        let result = self
            .application_service
            .count_branches_in_repositories(self.client_options.repository_urls().clone())
            .await;
        match result {
            Ok(counts_map) => {
                let counts: CountBranchesResult = counts_map.into();
                println!("{}", counts);
            }
            Err(err) => {
                eprintln!("Error: {}", err);
                std::process::exit(1)
            }
        };
    }

    pub async fn run(&self) {
        self.count_branches_in_repositories().await;
    }
}
