use ports::cli::adapters::ClientOptions;
use ports::repository_hosting::adapters::RepositoryHost;

use crate::application::{ApplicationService, RepositoryHostError};
use crate::cli_results::CountBranchesResult;

pub mod application;
mod cli_results;
pub mod ports;
pub mod utils;

pub struct TidyRepoClient<CO, RepoClient>
where
    CO: ClientOptions,
    RepoClient: RepositoryHost,
    <RepoClient as RepositoryHost>::Err: Into<RepositoryHostError>,
{
    client_options: CO,
    application_service: ApplicationService<RepoClient>,
}

impl<CO, RepoClient> TidyRepoClient<CO, RepoClient>
where
    CO: ClientOptions,
    RepoClient: RepositoryHost,
    <RepoClient as RepositoryHost>::Err: Into<RepositoryHostError>,
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
