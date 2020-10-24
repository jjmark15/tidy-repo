use structopt::StructOpt;

use tidy_repo::application::ApplicationService;
use tidy_repo::ports::cli::structopt::StructOptClientOptions;
use tidy_repo::ports::repository_hosting::github::{GitHubClient, GitHubRepositoryUrlParserImpl};
use tidy_repo::utils::environment::EnvironmentReaderStd;
use tidy_repo::utils::http::HttpClientFacadeImpl;
use tidy_repo::TidyRepoClient;

#[async_std::main]
async fn main() {
    let client_options = StructOptClientOptions::from_args();
    TidyRepoClient::new(client_options, application_service())
        .run()
        .await;
}

fn application_service() -> ApplicationService<
    GitHubClient<HttpClientFacadeImpl, GitHubRepositoryUrlParserImpl, EnvironmentReaderStd>,
> {
    let http_client = HttpClientFacadeImpl::new(surf::client());
    let url_parser = GitHubRepositoryUrlParserImpl::new();
    ApplicationService::new(GitHubClient::new(
        http_client,
        url_parser,
        EnvironmentReaderStd::new(),
    ))
}
