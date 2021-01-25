use std::path::PathBuf;

use structopt::StructOpt;

use std::process::exit;
use tidy_repo::application::ApplicationService;
use tidy_repo::domain::authentication::GitHubAuthenticationService;
use tidy_repo::domain::count_branches::BranchCounterServiceImpl;
use tidy_repo::ports::cli::terminal_client::{
    StructOptClientOptions, TerminalClientTidyRepoAppAdapter,
};
use tidy_repo::ports::persistence::filesystem::{
    FilesystemCredentialRepositoryAdapter, SerializableContentFilesystemStore,
};
use tidy_repo::ports::persistence::Credentials;
use tidy_repo::ports::repository_hosting::github::{
    GitHubClient, GitHubCredentialsValidatorAdapter, GitHubRepositoryProviderAdapter,
    GitHubRepositoryUrlParserImpl,
};
use tidy_repo::utils::environment::{EnvironmentReader, EnvironmentReaderStd};
use tidy_repo::utils::http::HttpClientFacadeImpl;
use tidy_repo::TidyRepoApp;

type GitHubClientAlias =
    GitHubClient<HttpClientFacadeImpl, GitHubRepositoryUrlParserImpl, EnvironmentReaderStd>;
type GitHubAuthenticationServiceAlias = GitHubAuthenticationService<
    GitHubCredentialsValidatorAdapter<GitHubClientAlias>,
    FilesystemCredentialRepositoryAdapterAlias,
>;
type FilesystemCredentialRepositoryAdapterAlias =
    FilesystemCredentialRepositoryAdapter<SerializableContentFilesystemStore<Credentials>>;

#[async_std::main]
async fn main() {
    tidy_repo_app().run().await;
}

fn tidy_repo_app() -> impl TidyRepoApp {
    let client_options = StructOptClientOptions::from_args();
    TerminalClientTidyRepoAppAdapter::new(client_options, application_service())
}

fn app_credentials_filepath() -> PathBuf {
    let env_reader = EnvironmentReaderStd::new();

    match &env_reader.read("TIDY_REPO_HOME") {
        Ok(path_string) => PathBuf::from(shellexpand::tilde(path_string).to_string()),
        Err(_e) => {
            eprintln!("Error: TIDY_REPO_HOME environment variable is not set");
            exit(1);
        }
    }
    .join("credentials.yml")
}

fn github_client() -> GitHubClientAlias {
    let http_client = HttpClientFacadeImpl::new(surf::client());
    let url_parser = GitHubRepositoryUrlParserImpl::new();
    GitHubClient::new(http_client, url_parser, EnvironmentReaderStd::new())
}

fn credential_repository() -> FilesystemCredentialRepositoryAdapterAlias {
    FilesystemCredentialRepositoryAdapter::new(SerializableContentFilesystemStore::new(
        app_credentials_filepath(),
    ))
}

fn github_authentication_service() -> GitHubAuthenticationServiceAlias {
    GitHubAuthenticationService::new(
        GitHubCredentialsValidatorAdapter::new(github_client()),
        credential_repository(),
    )
}

fn application_service() -> ApplicationService<
    BranchCounterServiceImpl,
    GitHubAuthenticationServiceAlias,
    GitHubRepositoryProviderAdapter<GitHubClientAlias, FilesystemCredentialRepositoryAdapterAlias>,
> {
    let github_repository_provider =
        GitHubRepositoryProviderAdapter::new(github_client(), credential_repository());
    let branch_counter_service = BranchCounterServiceImpl::new();
    ApplicationService::new(
        branch_counter_service,
        github_authentication_service(),
        github_repository_provider,
    )
}
