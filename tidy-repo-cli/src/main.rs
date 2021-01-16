use structopt::StructOpt;

use tidy_repo::application::ApplicationService;
use tidy_repo::domain::authentication::GitHubAuthenticationService;
use tidy_repo::domain::count_branches::BranchCounterServiceImpl;
use tidy_repo::ports::cli::structopt::StructOptClientOptions;
use tidy_repo::ports::persistence::filesystem::{
    FileSystemCredentialsPersistence, FilesystemAuthenticationPersistenceService,
};
use tidy_repo::ports::repository_hosting::github::{
    GitHubAuthenticationValidatorAdapter, GitHubClient, GitHubRepositoryProviderAdapter,
    GitHubRepositoryUrlParserImpl,
};
use tidy_repo::utils::environment::EnvironmentReaderStd;
use tidy_repo::utils::http::HttpClientFacadeImpl;
use tidy_repo::TidyRepoClient;

type GitHubClientAlias =
    GitHubClient<HttpClientFacadeImpl, GitHubRepositoryUrlParserImpl, EnvironmentReaderStd>;
type GitHubAuthenticationServiceAlias = GitHubAuthenticationService<
    GitHubAuthenticationValidatorAdapter<GitHubClientAlias>,
    FilesystemAuthenticationPersistenceServiceAlias,
>;
type FilesystemAuthenticationPersistenceServiceAlias = FilesystemAuthenticationPersistenceService<
    FileSystemCredentialsPersistence<EnvironmentReaderStd>,
>;

#[async_std::main]
async fn main() {
    let client_options = StructOptClientOptions::from_args();
    TidyRepoClient::new(client_options, application_service())
        .run()
        .await;
}

fn github_client() -> GitHubClientAlias {
    let http_client = HttpClientFacadeImpl::new(surf::client());
    let url_parser = GitHubRepositoryUrlParserImpl::new();
    GitHubClient::new(http_client, url_parser, EnvironmentReaderStd::new())
}

fn authentication_persistence_service() -> FilesystemAuthenticationPersistenceServiceAlias {
    FilesystemAuthenticationPersistenceService::new(FileSystemCredentialsPersistence::new(
        EnvironmentReaderStd::new(),
    ))
}

fn github_authentication_service() -> GitHubAuthenticationServiceAlias {
    GitHubAuthenticationService::new(
        GitHubAuthenticationValidatorAdapter::new(github_client()),
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
