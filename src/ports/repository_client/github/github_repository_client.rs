use std::collections::HashMap;

use async_trait::async_trait;
use http_types::headers::HeaderName;
use http_types::{Method, Url};

use crate::adapters::repository_client::RepositoryClient;
use crate::application::{BranchNameDto, RepositoryUrlDto};
use crate::ports::repository_client::github::parse_repository_url::GitHubRepositoryUrlParser;
use crate::ports::repository_client::github::responses::ListBranchesResponseBody;
use crate::ports::repository_client::github::GithubRepositoryClientError;
use crate::utils::environment::EnvironmentReader;
use crate::utils::http::{HttpClientFacade, Request};

#[derive(Debug)]
pub struct GitHubRepositoryClient<
    HttpClient: HttpClientFacade,
    UrlParser: GitHubRepositoryUrlParser,
    EnvReader: EnvironmentReader,
> {
    http_client: HttpClient,
    url_parser: UrlParser,
    environment_reader: EnvReader,
}

impl<HttpClient, UrlParser, EnvReader> GitHubRepositoryClient<HttpClient, UrlParser, EnvReader>
where
    HttpClient: HttpClientFacade,
    UrlParser: GitHubRepositoryUrlParser,
    EnvReader: EnvironmentReader,
{
    pub fn new(
        http_client: HttpClient,
        url_parser: UrlParser,
        environment_reader: EnvReader,
    ) -> Self {
        GitHubRepositoryClient {
            http_client,
            url_parser,
            environment_reader,
        }
    }

    fn list_branches_headers() -> HashMap<HeaderName, String> {
        let mut headers = HashMap::new();
        headers.insert(
            http_types::headers::ACCEPT,
            "application/vnd.github.v3+json".to_string(),
        );
        headers
    }

    fn parse_url(url_string: String) -> Result<Url, GithubRepositoryClientError> {
        match Url::parse(url_string.as_str()) {
            Ok(url) => Ok(url),
            Err(err) => Err(GithubRepositoryClientError::ApiUrlParseError(err)),
        }
    }

    fn api_base_url(&self) -> String {
        match self
            .environment_reader
            .read("TIDY_REPO_GITHUB_API_BASE_URL")
        {
            Ok(env_value) => env_value,
            Err(_) => "https://api.github.com".to_string(),
        }
    }

    fn list_branches_api_url(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<Url, GithubRepositoryClientError> {
        let url_string = format!("{}/repos/{}/{}/branches", self.api_base_url(), owner, repo);
        Self::parse_url(url_string)
    }
}

#[async_trait]
impl<HttpClient, UrlParser, EnvReader> RepositoryClient
    for GitHubRepositoryClient<HttpClient, UrlParser, EnvReader>
where
    HttpClient: HttpClientFacade + Send + Sync,
    UrlParser: GitHubRepositoryUrlParser + Send + Sync,
    EnvReader: EnvironmentReader + Send + Sync,
{
    type Err = GithubRepositoryClientError;

    async fn list_branches(
        &self,
        repository_url: &RepositoryUrlDto,
    ) -> Result<Vec<BranchNameDto>, Self::Err> {
        let repository = self.url_parser.parse(repository_url.clone())?;
        let response = self
            .http_client
            .send(Request::new(
                Method::Get,
                self.list_branches_api_url(repository.owner(), repository.name())?,
                Self::list_branches_headers(),
            ))
            .await?;
        match response.status_code() {
            http_types::StatusCode::Ok => Ok(response
                .body_json::<ListBranchesResponseBody>()?
                .branches()
                .iter()
                .map(|branch| BranchNameDto::new(branch.name().to_string()))
                .collect()),
            _ => Err(GithubRepositoryClientError::RepositoryNotFound(
                repository_url.clone(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::env::VarError;

    use http_types::{Method, StatusCode};
    use mockall::predicate::eq;
    use spectral::prelude::*;

    use crate::ports::repository_client::github::parse_repository_url::MockGitHubRepositoryUrlParser;
    use crate::ports::repository_client::github::repository::GitHubRepository;
    use crate::ports::repository_client::github::responses::{Branch, ListBranchesResponseBody};
    use crate::utils::environment::{EnvironmentReaderError, MockEnvironmentReader};
    use crate::utils::http::{Error, MockHttpClientFacade, Request, Response};

    use super::*;

    fn mock_http_client() -> MockHttpClientFacade {
        MockHttpClientFacade::default()
    }

    fn mock_repository_url_parser() -> MockGitHubRepositoryUrlParser {
        MockGitHubRepositoryUrlParser::default()
    }

    fn mock_environment_reader() -> MockEnvironmentReader {
        let mut reader = MockEnvironmentReader::default();
        reader
            .expect_read()
            .returning(|_| Err(EnvironmentReaderError::ReadError(VarError::NotPresent)));
        reader
    }

    async fn wrap_response_in_future(response: Result<Response, Error>) -> Result<Response, Error> {
        response
    }

    fn prepare_mock_http_client(
        mock_http_client: &mut MockHttpClientFacade,
        request: Request,
        response: Response,
    ) {
        mock_http_client
            .expect_send()
            .with(eq(request))
            .returning(move |_| Box::pin(wrap_response_in_future(Ok(response.clone()))));
    }

    fn fake_valid_url(owner: &str, repo: &str) -> RepositoryUrlDto {
        RepositoryUrlDto::new(format!(
            "https://api.github.com/repos/{}/{}/branches",
            owner, repo
        ))
    }

    fn valid_list_branches_request(owner: &str, repo: &str) -> Request {
        let mut headers = HashMap::new();
        headers.insert(
            http_types::headers::ACCEPT,
            "application/vnd.github.v3+json".to_string(),
        );
        Request::new(
            Method::Get,
            fake_valid_url(owner, repo).value().parse().unwrap(),
            headers,
        )
    }

    fn successful_list_branches_response(
        list_branches_response_body: ListBranchesResponseBody,
    ) -> Response {
        Response::new(
            StatusCode::Ok,
            serde_json::json!(list_branches_response_body).to_string(),
        )
    }

    #[async_std::test]
    async fn lists_branches() {
        let mut mock_http_client = mock_http_client();
        prepare_mock_http_client(
            &mut mock_http_client,
            valid_list_branches_request("owner", "repo"),
            successful_list_branches_response(ListBranchesResponseBody::new(vec![Branch::new(
                "branch".to_string(),
            )])),
        );
        let mut mock_url_parser = mock_repository_url_parser();
        mock_url_parser
            .expect_parse()
            .with(eq(RepositoryUrlDto::new(
                "https://github.com/owner/repo".to_string(),
            )))
            .returning(|_| {
                Ok(GitHubRepository::new(
                    "owner".to_string(),
                    "repo".to_string(),
                ))
            });

        let under_test = GitHubRepositoryClient::new(
            mock_http_client,
            mock_url_parser,
            mock_environment_reader(),
        );

        assert_that(
            &under_test
                .list_branches(&RepositoryUrlDto::new(
                    "https://github.com/owner/repo".to_string(),
                ))
                .await
                .unwrap(),
        )
        .is_equal_to(&vec![BranchNameDto::new("branch".to_string())]);
    }
}
