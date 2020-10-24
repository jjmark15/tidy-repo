use ports::cli::adapters::ClientOptions;
use ports::cli::commands::CliCommand;

use crate::application::ApplicationService;
use crate::cli_results::CountBranchesResult;
use crate::domain::count_branches::BranchCounterService;

pub mod application;
mod cli_results;
pub mod domain;
pub mod ports;
pub mod utils;

pub struct TidyRepoClient<CO, BranchCounter>
where
    CO: ClientOptions,
    BranchCounter: BranchCounterService,
{
    client_options: CO,
    application_service: ApplicationService<BranchCounter>,
}

impl<CO, BranchCounter> TidyRepoClient<CO, BranchCounter>
where
    CO: ClientOptions,
    BranchCounter: BranchCounterService,
{
    pub fn new(client_options: CO, application_service: ApplicationService<BranchCounter>) -> Self {
        TidyRepoClient {
            client_options,
            application_service,
        }
    }

    async fn count_branches_in_repositories(&self) {
        let result = self
            .application_service
            .count_branches_in_repositories(self.client_options.repository_urls().unwrap().clone())
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
        match self.client_options.command() {
            CliCommand::Branches => self.count_branches_in_repositories().await,
        }
    }
}
