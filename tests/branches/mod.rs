use std::path::Path;

use assert_cmd::Command;
use http_types::headers::ACCEPT;
use http_types::Method;

use crate::common::test_command;

mod counts_branches_in_multiple_github_repositories;
mod counts_branches_in_private_github_repository;
mod counts_branches_in_single_github_repository;
mod fails_to_find_private_repositories_when_not_authenticated;
mod fails_when_app_home_environment_variable_is_not_set;
mod fails_when_passed_a_malformed_repository_url;
mod fails_when_passed_a_repository_url_that_does_not_exist;
mod fails_when_passed_repository_url_missing_owner;
mod fails_when_passed_repository_url_missing_repo_name;
mod returns_empty_results_when_not_passed_any_repository_urls;

pub(crate) fn mock_github_api_server_for_successful_list_branches(
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
    .with_status(200)
}

pub(crate) fn count_results_with_header<S: AsRef<str>>(count_results: S) -> String {
    count_results.as_ref().to_string()
}

pub(crate) fn branches_command(temp_home_directory: &Path) -> Command {
    let mut cmd = test_command(temp_home_directory);
    cmd.arg("branches");
    cmd
}
