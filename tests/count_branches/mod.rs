use http_types::headers::ACCEPT;
use http_types::Method;

mod counts_branches_in_multiple_github_repositories;
mod counts_branches_in_single_github_repository;
mod errors_when_passed_a_repository_url_that_does_not_exist;
mod errors_when_passed_repository_url_missing_owner;
mod errors_when_passed_repository_url_missing_repo_name;
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
    .create()
}
