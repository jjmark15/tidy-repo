use std::path::Path;

use assert_cmd::Command;
use http_types::Method;

use crate::common::test_command;

mod authenticates_with_github;
mod authentication_with_github_overwrites_previous_github_authentication;
mod fails_to_authenticate_with_github_when_app_home_directory_does_not_exist;
mod fails_to_authenticate_with_github_when_passed_invalid_token;

pub(crate) fn authenticate_command(temp_home_directory: &Path) -> Command {
    let mut cmd = test_command(temp_home_directory);
    cmd.arg("authenticate");
    cmd
}

pub(crate) fn mock_github_api_server_for_successful_authentication_check(
    oauth_token: &str,
) -> mockito::Mock {
    mockito::mock(Method::Get.as_ref(), "/")
        .match_header(
            http_types::headers::AUTHORIZATION.as_str(),
            format!("token {}", oauth_token).as_str(),
        )
        .with_status(200)
}

pub(crate) fn mock_github_api_server_for_unsuccessful_authentication_check(
    oauth_token: &str,
) -> mockito::Mock {
    mockito::mock(Method::Get.as_ref(), "/")
        .match_header(
            http_types::headers::AUTHORIZATION.as_str(),
            format!("token {}", oauth_token).as_str(),
        )
        .with_status(401)
}
