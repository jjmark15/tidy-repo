use std::path::Path;

use assert_cmd::output::OutputResult;
use http_types::headers::{ACCEPT, AUTHORIZATION};
use http_types::Method;

use crate::authenticate::{
    authenticate_command, mock_github_api_server_for_successful_authentication_check,
};

pub(crate) fn test_command(temp_home_directory: &Path) -> assert_cmd::Command {
    let mut cmd = assert_cmd::Command::cargo_bin("tidy-repo")
        .expect("Could not run cargo binary 'tidy-repo'");
    cmd.env(GITHUB_API_URL, mockito::server_url()).env(
        APP_HOME_ENVIRONMENT_VARIABLE,
        temp_home_directory.as_os_str(),
    );
    cmd
}

pub(crate) fn require_github_auth_for_mock(mock: mockito::Mock) -> mockito::Mock {
    mock.match_header(
        AUTHORIZATION.as_str(),
        format!("token {}", GITHUB_OAUTH_TOKEN).as_str(),
    )
}

pub(crate) fn authenticate_session_with_github(app_home_directory_path: &Path) -> OutputResult {
    authenticate_session_with_github_with_token(app_home_directory_path, GITHUB_OAUTH_TOKEN)
}

pub(crate) fn authenticate_session_with_github_with_token(
    app_home_directory_path: &Path,
    token: &str,
) -> OutputResult {
    let _mock = mock_github_api_server_for_successful_authentication_check(token).create();
    authenticate_command(app_home_directory_path)
        .arg("github")
        .arg("--token")
        .arg(token)
        .ok()
}

pub(crate) fn mock_github_api_server_for_repository_not_found(
    owner: &str,
    repo_name: &str,
    body: &str,
) -> mockito::Mock {
    mockito::mock(
        Method::Get.as_ref(),
        format!("/repos/{}/{}/branches", owner, repo_name).as_str(),
    )
    .match_header(ACCEPT.as_str(), "application/vnd.github.v3+json")
    .with_body(body)
    .with_status(404)
}

pub const GITHUB_OAUTH_TOKEN: &str = "OAUTH-TOKEN";
pub const GITHUB_API_URL: &str = "TIDY_REPO_GITHUB_API_BASE_URL";
pub const APP_HOME_ENVIRONMENT_VARIABLE: &str = "TIDY_REPO_HOME";
