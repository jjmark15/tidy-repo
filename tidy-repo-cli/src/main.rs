use structopt::StructOpt;

use tidy_repo::application::ApplicationService;
use tidy_repo::domain::authentication::{GitHubAuthenticationService, GitHubAuthenticationToken};
use tidy_repo::domain::authentication_persistence::PersistAuthenticationImpl;
use tidy_repo::domain::count_branches::BranchCounterServiceImpl;
use tidy_repo::domain::repository_host::{
    AuthenticatedRepositoryHostWrapper, UnauthenticatedRepositoryHostWrapper,
};
use tidy_repo::ports::cli::adapters::structopt::StructOptClientOptions;
use tidy_repo::ports::persistence::adapters::filesystem::CredentialsFileSystemPersistenceService;
use tidy_repo::ports::repository_hosting::adapters::github::{
    GitHubClient, GitHubRepositoryUrlParserImpl,
};
use tidy_repo::utils::environment::EnvironmentReaderStd;
use tidy_repo::utils::http::HttpClientFacadeImpl;
use tidy_repo::TidyRepoClient;

type GitHubClientAlias =
    GitHubClient<HttpClientFacadeImpl, GitHubRepositoryUrlParserImpl, EnvironmentReaderStd>;
type GitHubAuthenticationServiceAlias = GitHubAuthenticationService<
    UnauthenticatedRepositoryHostWrapper<GitHubClientAlias, GitHubAuthenticationToken>,
    PersistAuthenticationImpl<
        GitHubAuthenticationToken,
        CredentialsFileSystemPersistenceService<EnvironmentReaderStd>,
    >,
>;
type BranchCounterServiceAlias = BranchCounterServiceImpl<
    AuthenticatedRepositoryHostWrapper<
        GitHubClientAlias,
        GitHubAuthenticationServiceAlias,
        GitHubAuthenticationToken,
    >,
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

fn github_authentication_service() -> GitHubAuthenticationServiceAlias {
    let environment_reader = EnvironmentReaderStd::new();
    let unauthenticated_github_repository_host_wrapper =
        UnauthenticatedRepositoryHostWrapper::new(github_client());
    let github_authentication_persistence = PersistAuthenticationImpl::new(
        CredentialsFileSystemPersistenceService::new(environment_reader),
    );
    GitHubAuthenticationService::new(
        unauthenticated_github_repository_host_wrapper,
        github_authentication_persistence,
    )
}

fn application_service(
) -> ApplicationService<BranchCounterServiceAlias, GitHubAuthenticationServiceAlias> {
    let authenticated_github_repository_host_wrapper =
        AuthenticatedRepositoryHostWrapper::new(github_client(), github_authentication_service());
    let branch_counter_service =
        BranchCounterServiceImpl::new(authenticated_github_repository_host_wrapper);
    ApplicationService::new(branch_counter_service, github_authentication_service())
}
