use std::collections::HashMap;

use async_trait::async_trait;
use http_types::headers::HeaderName;
use http_types::{Method, Url};

use crate::ports::repository_hosting::github::authentication_token::GitHubAuthenticationToken;
use crate::ports::repository_hosting::github::error::GitHubClientError;
use crate::ports::repository_hosting::github::repository::{BranchName, RepositoryUrl};
use crate::ports::repository_hosting::github::{
    parse_repository_url::GitHubRepositoryUrlParser, responses::ListBranchesResponseBody,
    AuthenticationCredentialValidity,
};
use crate::utils::environment::EnvironmentReader;
use crate::utils::http::{HttpClientFacade, Request};

#[async_trait]
pub trait RepositoryHostClient {
    type Err;
    type AuthenticationCredentials;

    async fn list_branches(
        &self,
        repository_url: &RepositoryUrl,
    ) -> Result<Vec<BranchName>, Self::Err>;

    fn set_authentication_credentials(&mut self, credentials: Self::AuthenticationCredentials);

    async fn validate_authentication_credentials(
        &self,
        credentials: Self::AuthenticationCredentials,
    ) -> Result<AuthenticationCredentialValidity, Self::Err>;
}

#[cfg(test)]
mockall::mock! {
    pub RepositoryHostClient<Err: 'static + Send + Sync, C: 'static + Send + Sync> {}

    #[async_trait::async_trait]
    impl<Err: 'static + Send + Sync, C: 'static + Send + Sync> RepositoryHostClient for RepositoryClient<Err, C> {
        type Err = Err;
        type AuthenticationCredentials = C;

        async fn list_branches(
            &self,
            repository_url: &RepositoryUrl,
        ) -> Result<Vec<BranchName>, Err>;

        fn set_authentication_credentials(&mut self, credentials: C);

        async fn validate_authentication_credentials(
            &self,
            credentials: C,
        ) -> Result<AuthenticationCredentialValidity, Err>;
    }
}

#[derive(Debug)]
pub struct GitHubClient<
    HttpClient: HttpClientFacade,
    UrlParser: GitHubRepositoryUrlParser,
    EnvReader: EnvironmentReader,
> {
    http_client: HttpClient,
    url_parser: UrlParser,
    environment_reader: EnvReader,
    personal_access_token: Option<GitHubAuthenticationToken>,
}

impl<HttpClient, UrlParser, EnvReader> GitHubClient<HttpClient, UrlParser, EnvReader>
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
        GitHubClient {
            http_client,
            url_parser,
            environment_reader,
            personal_access_token: None,
        }
    }

    fn list_branches_headers(&self) -> HashMap<HeaderName, String> {
        let mut headers = HashMap::new();
        headers.insert(
            http_types::headers::ACCEPT,
            "application/vnd.github.v3+json".to_string(),
        );
        if let Some(authentication_token) = &self.personal_access_token {
            headers.insert(
                http_types::headers::AUTHORIZATION,
                format!("token {}", authentication_token.value()),
            );
        }
        headers
    }

    fn validate_authentication_credentials_headers(
        &self,
        token: GitHubAuthenticationToken,
    ) -> HashMap<HeaderName, String> {
        let mut headers = HashMap::new();
        headers.insert(
            http_types::headers::AUTHORIZATION,
            format!("token {}", token.value()),
        );
        headers
    }

    fn parse_url(url_string: String) -> Result<Url, GitHubClientError> {
        match Url::parse(url_string.as_str()) {
            Ok(url) => Ok(url),
            Err(err) => Err(GitHubClientError::ApiUrlParseError(err)),
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

    fn list_branches_api_url(&self, owner: &str, repo: &str) -> Result<Url, GitHubClientError> {
        let url_string = format!("{}/repos/{}/{}/branches", self.api_base_url(), owner, repo);
        Self::parse_url(url_string)
    }

    fn validate_authentication_credentials_api_url(&self) -> Result<Url, GitHubClientError> {
        Self::parse_url(self.api_base_url())
    }
}

#[async_trait]
impl<HttpClient, UrlParser, EnvReader> RepositoryHostClient
    for GitHubClient<HttpClient, UrlParser, EnvReader>
where
    HttpClient: HttpClientFacade + Send + Sync,
    UrlParser: GitHubRepositoryUrlParser + Send + Sync,
    EnvReader: EnvironmentReader + Send + Sync,
{
    type Err = GitHubClientError;
    type AuthenticationCredentials = GitHubAuthenticationToken;

    async fn list_branches(
        &self,
        repository_url: &RepositoryUrl,
    ) -> Result<Vec<BranchName>, Self::Err> {
        let repository = self.url_parser.parse(repository_url.clone())?;

        let response = self
            .http_client
            .send(Request::new(
                Method::Get,
                self.list_branches_api_url(repository.owner(), repository.name())?,
                self.list_branches_headers(),
            ))
            .await?;

        match response.status_code() {
            http_types::StatusCode::Ok => Ok(response
                .body_json::<ListBranchesResponseBody>()?
                .branches()
                .iter()
                .map(|branch| BranchName::new(branch.name().to_string()))
                .collect()),
            _ => Err(GitHubClientError::RepositoryNotFound(
                repository_url.clone(),
            )),
        }
    }

    fn set_authentication_credentials(&mut self, credentials: Self::AuthenticationCredentials) {
        self.personal_access_token = Some(credentials);
    }

    async fn validate_authentication_credentials(
        &self,
        credentials: Self::AuthenticationCredentials,
    ) -> Result<AuthenticationCredentialValidity, Self::Err> {
        let response = self
            .http_client
            .send(Request::new(
                Method::Get,
                self.validate_authentication_credentials_api_url()?,
                self.validate_authentication_credentials_headers(credentials),
            ))
            .await?;

        match response.status_code() {
            http_types::StatusCode::Ok => Ok(AuthenticationCredentialValidity::Valid),
            http_types::StatusCode::Unauthorized => Ok(AuthenticationCredentialValidity::Invalid),
            _ => Err(GitHubClientError::Unexpected),
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

    use crate::ports::repository_hosting::github::repository::GitHubRepository;
    use crate::ports::repository_hosting::github::responses::Branch;
    use crate::ports::repository_hosting::github::MockGitHubRepositoryUrlParser;
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

    fn fake_list_branches_api_url(owner: &str, repo: &str) -> Url {
        Url::parse(format!("https://api.github.com/repos/{}/{}/branches", owner, repo).as_str())
            .unwrap()
    }

    fn fake_validate_authentication_credentials_api_url() -> Url {
        Url::parse("https://api.github.com/").unwrap()
    }

    fn valid_list_branches_request(owner: &str, repo: &str) -> Request {
        let mut headers = HashMap::new();
        headers.insert(
            http_types::headers::ACCEPT,
            "application/vnd.github.v3+json".to_string(),
        );
        Request::new(
            Method::Get,
            fake_list_branches_api_url(owner, repo),
            headers,
        )
    }

    fn authenticated_list_branches_request(
        owner: &str,
        repo: &str,
        authentication_token: GitHubAuthenticationToken,
    ) -> Request {
        let mut headers = HashMap::new();
        headers.insert(
            http_types::headers::ACCEPT,
            "application/vnd.github.v3+json".to_string(),
        );
        headers.insert(
            http_types::headers::AUTHORIZATION,
            format!("token {}", authentication_token.value()),
        );
        Request::new(
            Method::Get,
            fake_list_branches_api_url(owner, repo),
            headers,
        )
    }

    fn valid_validate_authentication_credentials_request(
        token: GitHubAuthenticationToken,
    ) -> Request {
        let mut headers = HashMap::new();
        headers.insert(
            http_types::headers::AUTHORIZATION,
            format!("token {}", token.value()),
        );

        Request::new(
            Method::Get,
            fake_validate_authentication_credentials_api_url(),
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

    fn validate_authentication_credentials_response(
        status_code: http_types::StatusCode,
    ) -> Response {
        Response::new(status_code, "".to_string())
    }

    #[test]
    fn auth_token_is_absent_by_default() {
        let under_test = GitHubClient::new(
            mock_http_client(),
            mock_repository_url_parser(),
            mock_environment_reader(),
        );

        assert_that(&under_test.personal_access_token).is_none();
    }

    #[test]
    fn sets_authentication_token() {
        let mut under_test = GitHubClient::new(
            mock_http_client(),
            mock_repository_url_parser(),
            mock_environment_reader(),
        );

        under_test
            .set_authentication_credentials(GitHubAuthenticationToken::new("token".to_string()));

        assert_that(&under_test.personal_access_token.unwrap())
            .is_equal_to(GitHubAuthenticationToken::new("token".to_string()));
    }

    #[async_std::test]
    async fn lists_branches_without_authentication_credentials_set() {
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
            .with(eq(RepositoryUrl::new(
                "https://github.com/owner/repo".to_string(),
            )))
            .returning(|_| {
                Ok(GitHubRepository::new(
                    "owner".to_string(),
                    "repo".to_string(),
                ))
            });

        let under_test =
            GitHubClient::new(mock_http_client, mock_url_parser, mock_environment_reader());

        assert_that(
            &under_test
                .list_branches(&RepositoryUrl::new(
                    "https://github.com/owner/repo".to_string(),
                ))
                .await
                .unwrap(),
        )
        .is_equal_to(&vec![BranchName::new("branch".to_string())]);
    }

    #[async_std::test]
    async fn lists_branches_with_authentication_credentials_set() {
        let mut mock_http_client = mock_http_client();
        prepare_mock_http_client(
            &mut mock_http_client,
            authenticated_list_branches_request(
                "owner",
                "repo",
                GitHubAuthenticationToken::new("token".to_string()),
            ),
            successful_list_branches_response(ListBranchesResponseBody::new(vec![Branch::new(
                "branch".to_string(),
            )])),
        );
        let mut mock_url_parser = mock_repository_url_parser();
        mock_url_parser
            .expect_parse()
            .with(eq(RepositoryUrl::new(
                "https://github.com/owner/repo".to_string(),
            )))
            .returning(|_| {
                Ok(GitHubRepository::new(
                    "owner".to_string(),
                    "repo".to_string(),
                ))
            });

        let mut under_test =
            GitHubClient::new(mock_http_client, mock_url_parser, mock_environment_reader());
        under_test
            .set_authentication_credentials(GitHubAuthenticationToken::new("token".to_string()));

        assert_that(
            &under_test
                .list_branches(&RepositoryUrl::new(
                    "https://github.com/owner/repo".to_string(),
                ))
                .await
                .unwrap(),
        )
        .is_equal_to(&vec![BranchName::new("branch".to_string())]);
    }

    #[async_std::test]
    async fn validates_valid_authentication_credentials() {
        let token = GitHubAuthenticationToken::new("token".to_string());
        let mut mock_http_client = mock_http_client();
        prepare_mock_http_client(
            &mut mock_http_client,
            valid_validate_authentication_credentials_request(token.clone()),
            validate_authentication_credentials_response(http_types::StatusCode::Ok),
        );
        let mock_url_parser = mock_repository_url_parser();

        let under_test =
            GitHubClient::new(mock_http_client, mock_url_parser, mock_environment_reader());

        assert_that(
            &under_test
                .validate_authentication_credentials(token)
                .await
                .unwrap(),
        )
        .is_equal_to(AuthenticationCredentialValidity::Valid);
    }

    #[async_std::test]
    async fn validates_invalid_authentication_credentials() {
        let token = GitHubAuthenticationToken::new("token".to_string());
        let mut mock_http_client = mock_http_client();
        prepare_mock_http_client(
            &mut mock_http_client,
            valid_validate_authentication_credentials_request(token.clone()),
            validate_authentication_credentials_response(http_types::StatusCode::Unauthorized),
        );
        let mock_url_parser = mock_repository_url_parser();

        let under_test =
            GitHubClient::new(mock_http_client, mock_url_parser, mock_environment_reader());

        assert_that(
            &under_test
                .validate_authentication_credentials(token)
                .await
                .unwrap(),
        )
        .is_equal_to(AuthenticationCredentialValidity::Invalid);
    }
}
