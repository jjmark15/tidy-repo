use ports::cli::commands::CliCommand;
use ports::cli::ClientOptions;

use crate::application::ApplicationService;
use crate::cli_results::CountBranchesResult;
use crate::domain::authentication::AuthenticationService;
use crate::domain::authentication::GitHubAuthenticationToken as DomainCliGitHubAuthenticationToken;
use crate::domain::count_branches::BranchCounterService;
use crate::domain::repository::RepositoryProvider;
use crate::ports::cli::GitHubAuthenticationToken;

pub mod application;
mod cli_results;
pub mod domain;
pub mod ports;
pub mod utils;

pub struct TidyRepoClient<CO, BranchCounter, GAS, GRP>
where
    CO: ClientOptions,
    BranchCounter: BranchCounterService,
    GAS: AuthenticationService<AuthenticationCredentials = DomainCliGitHubAuthenticationToken>,
    GRP: RepositoryProvider,
{
    client_options: CO,
    application_service: ApplicationService<BranchCounter, GAS, GRP>,
}

impl<CO, BranchCounter, GAS, GRP> TidyRepoClient<CO, BranchCounter, GAS, GRP>
where
    CO: ClientOptions,
    BranchCounter: BranchCounterService,
    GAS: AuthenticationService<AuthenticationCredentials = DomainCliGitHubAuthenticationToken>,
    GRP: RepositoryProvider,
{
    pub fn new(
        client_options: CO,
        application_service: ApplicationService<BranchCounter, GAS, GRP>,
    ) -> Self {
        TidyRepoClient {
            client_options,
            application_service,
        }
    }

    async fn count_branches_in_repositories(&mut self) {
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

    async fn authenticate_github(&self, github_token: GitHubAuthenticationToken) {
        let result = self
            .application_service
            .authenticate_app_with_github(github_token)
            .await;
        match result {
            Ok(_) => println!("Successfully authenticated with GitHub"),
            Err(err) => {
                eprintln!("Error: {}", err);
                std::process::exit(1)
            }
        }
    }

    pub async fn run(&mut self) {
        match self.client_options.command() {
            CliCommand::AuthenticateGitHub => {
                self.authenticate_github(self.client_options.github_auth_token().unwrap())
                    .await
            }
            CliCommand::Branches => self.count_branches_in_repositories().await,
        }
    }
}
