use crate::application::ApplicationService;
use crate::domain::authentication::{
    AuthenticationService, GitHubAuthenticationToken as DomainCliGitHubAuthenticationToken,
};
use crate::domain::count_branches::BranchCounterService;
use crate::domain::repository::RepositoryProvider;
use crate::ports::cli::terminal_client::cli_results::CountBranchesResult;
use crate::ports::cli::terminal_client::commands::CliCommand;
use crate::ports::cli::terminal_client::github_token::GitHubAuthenticationToken;
use crate::ports::cli::terminal_client::ClientOptions;
use crate::TidyRepoApp;

pub struct TerminalClientTidyRepoAppAdapter<CO, BranchCounter, GAS, GRP>
where
    CO: ClientOptions,
    BranchCounter: BranchCounterService,
    GAS: AuthenticationService<AuthenticationCredentials = DomainCliGitHubAuthenticationToken>,
    GRP: RepositoryProvider,
{
    client_options: CO,
    application_service: ApplicationService<BranchCounter, GAS, GRP>,
}

impl<CO, BranchCounter, GAS, GRP> TerminalClientTidyRepoAppAdapter<CO, BranchCounter, GAS, GRP>
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
        TerminalClientTidyRepoAppAdapter {
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
            .authenticate_app_with_github(github_token.value().to_string())
            .await;
        match result {
            Ok(_) => println!("Successfully authenticated with GitHub"),
            Err(err) => {
                eprintln!("Error: {}", err);
                std::process::exit(1)
            }
        }
    }
}

#[async_trait::async_trait]
impl<CO, BranchCounter, GAS, GRP> TidyRepoApp
    for TerminalClientTidyRepoAppAdapter<CO, BranchCounter, GAS, GRP>
where
    CO: ClientOptions + Send + Sync,
    BranchCounter: BranchCounterService + Send + Sync,
    GAS: AuthenticationService<AuthenticationCredentials = DomainCliGitHubAuthenticationToken>
        + Send
        + Sync,
    GRP: RepositoryProvider + Send + Sync,
{
    async fn run(&mut self) {
        match self.client_options.command() {
            CliCommand::AuthenticateGitHub => {
                self.authenticate_github(self.client_options.github_auth_token().unwrap())
                    .await
            }
            CliCommand::Branches => self.count_branches_in_repositories().await,
        }
    }
}
