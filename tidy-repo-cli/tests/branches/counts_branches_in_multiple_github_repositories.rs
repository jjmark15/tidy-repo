use crate::branches::{
    branches_command, count_results_with_header,
    mock_github_api_server_for_successful_list_branches,
};

#[test]
fn counts_branches_in_multiple_github_repositories() {
    let temp_home_directory = assert_fs::TempDir::new().unwrap();
    let repo_1_body_string = "[{\"name\": \"branch1\"}]";
    let _mock_1 =
        mock_github_api_server_for_successful_list_branches("owner", "repo1", repo_1_body_string)
            .create();
    let repo_2_body_string = "[{\"name\": \"branch2\"}]";
    let _mock_2 =
        mock_github_api_server_for_successful_list_branches("owner", "repo2", repo_2_body_string)
            .create();

    let assert = branches_command(temp_home_directory.path())
        .arg("https://github.com/owner/repo1")
        .arg("https://github.com/owner/repo2")
        .assert();

    assert.success().stdout(count_results_with_header(
        "https://github.com/owner/repo1: 1\nhttps://github.com/owner/repo2: 1\n",
    ));
    temp_home_directory.close().unwrap();
}
