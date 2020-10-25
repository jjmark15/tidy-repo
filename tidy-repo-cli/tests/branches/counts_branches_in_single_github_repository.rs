use crate::branches::{
    branches_command, count_results_with_header,
    mock_github_api_server_for_successful_list_branches,
};

#[test]
fn counts_branches_in_single_github_repository() {
    let body_string = "[{\"name\": \"branch\"}]";
    let _mock = mock_github_api_server_for_successful_list_branches("owner", "repo", body_string);

    let assert = branches_command()
        .arg("https://github.com/owner/repo")
        .assert();

    assert.success().stdout(count_results_with_header(
        "https://github.com/owner/repo: 1\n",
    ));
}

#[test]
fn counts_branches_in_single_github_repository_without_url_schema() {
    let body_string = "[{\"name\": \"branch\"}]";
    let _mock = mock_github_api_server_for_successful_list_branches("owner", "repo", body_string);

    let assert = branches_command().arg("github.com/owner/repo").assert();

    assert
        .success()
        .stdout(count_results_with_header("github.com/owner/repo: 1\n"));
}
