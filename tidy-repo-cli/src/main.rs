use structopt::StructOpt;

use tidy_repo::application::ApplicationService;
use tidy_repo::domain::authentication::GitHubAuthenticationService;
use tidy_repo::domain::count_branches::BranchCounterServiceImpl;
use tidy_repo::ports::initiation::terminal_client::{
    StructOptClientOptions, TerminalClientTidyRepoAppAdapter,
};
use tidy_repo::ports::persistence::filesystem::{
    FileSystemCredentialsPersistenceImpl, FilesystemCredentialRepositoryAdapter,
};
use tidy_repo::ports::repository_hosting::github::{
    GitHubClient, GitHubCredentialsValidatorAdapter, GitHubRepositoryProviderAdapter,
    GitHubRepositoryUrlParserImpl,
};
use tidy_repo::utils::environment::EnvironmentReaderStd;
use tidy_repo::utils::http::HttpClientFacadeImpl;
use tidy_repo::TidyRepoApp;

type GitHubClientAlias =
    GitHubClient<HttpClientFacadeImpl, GitHubRepositoryUrlParserImpl, EnvironmentReaderStd>;
type GitHubAuthenticationServiceAlias = GitHubAuthenticationService<
    GitHubCredentialsValidatorAdapter<GitHubClientAlias>,
    FilesystemAuthenticationPersistenceServiceAlias,
>;
type FilesystemAuthenticationPersistenceServiceAlias = FilesystemCredentialRepositoryAdapter<
    FileSystemCredentialsPersistenceImpl<EnvironmentReaderStd>,
>;

#[async_std::main]
async fn main() {
    tidy_repo_app().run().await;
}

fn tidy_repo_app() -> impl TidyRepoApp {
    let client_options = StructOptClientOptions::from_args();
    TerminalClientTidyRepoAppAdapter::new(client_options, application_service())
}

fn github_client() -> GitHubClientAlias {
    let http_client = HttpClientFacadeImpl::new(surf::client());
    let url_parser = GitHubRepositoryUrlParserImpl::new();
    GitHubClient::new(http_client, url_parser, EnvironmentReaderStd::new())
}

fn authentication_persistence_service() -> FilesystemAuthenticationPersistenceServiceAlias {
    FilesystemCredentialRepositoryAdapter::new(FileSystemCredentialsPersistenceImpl::new(
        EnvironmentReaderStd::new(),
    ))
}

fn github_authentication_service() -> GitHubAuthenticationServiceAlias {
    GitHubAuthenticationService::new(
        GitHubCredentialsValidatorAdapter::new(github_client()),
        authentication_persistence_service(),
    )
}

fn application_service() -> ApplicationService<
    BranchCounterServiceImpl,
    GitHubAuthenticationServiceAlias,
    GitHubRepositoryProviderAdapter<
        GitHubClientAlias,
        FilesystemAuthenticationPersistenceServiceAlias,
    >,
> {
    let github_repository_provider =
        GitHubRepositoryProviderAdapter::new(github_client(), authentication_persistence_service());
    let branch_counter_service = BranchCounterServiceImpl::new();
    ApplicationService::new(
        branch_counter_service,
        github_authentication_service(),
        github_repository_provider,
    )
}
