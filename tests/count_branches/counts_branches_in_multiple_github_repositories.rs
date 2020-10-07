use crate::common::test_command;

use super::mock_github_api_server_for_successful_list_branches;

#[test]
fn counts_branches_in_multiple_github_repositories() {
    let repo_1_body_string = "[{\"name\": \"branch1\"}]";
    let _mock_1 =
        mock_github_api_server_for_successful_list_branches("owner", "repo1", repo_1_body_string);
    let repo_2_body_string = "[{\"name\": \"branch2\"}]";
    let _mock_2 =
        mock_github_api_server_for_successful_list_branches("owner", "repo2", repo_2_body_string);
    let mut cmd = test_command();

    let assert = cmd
        .arg("https://github.com/owner/repo1")
        .arg("https://github.com/owner/repo2")
        .assert();

    assert
        .success()
        .stdout("https://github.com/owner/repo1: 1\nhttps://github.com/owner/repo2: 1\n");
}
