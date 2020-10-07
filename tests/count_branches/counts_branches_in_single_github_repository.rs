use crate::common::test_command;

use super::mock_github_api_server_for_successful_list_branches;

#[test]
fn counts_branches_in_single_github_repository() {
    let body_string = "[{\"name\": \"branch\"}]";
    let _mock = mock_github_api_server_for_successful_list_branches("owner", "repo", body_string);
    let mut cmd = test_command();

    let assert = cmd.arg("https://github.com/owner/repo").assert();

    assert
        .success()
        .stdout("https://github.com/owner/repo: 1\n");
}

#[test]
fn counts_branches_in_single_github_repository_without_url_schema() {
    let body_string = "[{\"name\": \"branch\"}]";
    let _mock = mock_github_api_server_for_successful_list_branches("owner", "repo", body_string);
    let mut cmd = test_command();

    let assert = cmd.arg("github.com/owner/repo").assert();

    assert.success().stdout("github.com/owner/repo: 1\n");
}
