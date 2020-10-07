use http_types::headers::ACCEPT;
use http_types::Method;

use crate::common::test_command;

#[test]
fn errors_when_passed_repository_url_that_does_not_exist() {
    let body_string = "[{\"message\": \"Not Found\"}]";
    let _mock = mock_github_api_server_for_repository_not_found("owner", "repo", body_string);
    let mut cmd = test_command();

    let assert = cmd.arg("https://github.com/owner/repo").assert();

    assert
        .failure()
        .stderr("Error: repository 'https://github.com/owner/repo' not found\n");
}

fn mock_github_api_server_for_repository_not_found(
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
    .create()
}
