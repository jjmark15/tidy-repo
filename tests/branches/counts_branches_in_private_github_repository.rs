use crate::branches::{
    branches_command, count_results_with_header,
    mock_github_api_server_for_successful_list_branches,
};
use crate::common::{authenticate_session_with_github, require_github_auth_for_mock};

#[test]
fn counts_branches_in_private_github_repository() {
    let temp_home_directory = assert_fs::TempDir::new().unwrap();
    let body_string = "[{\"name\": \"branch\"}]";
    let _mock = require_github_auth_for_mock(mock_github_api_server_for_successful_list_branches(
        "owner",
        "repo",
        body_string,
    ))
    .create();
    authenticate_session_with_github(temp_home_directory.path()).unwrap();

    let assert = branches_command(temp_home_directory.path())
        .arg("https://github.com/owner/repo")
        .assert();

    assert.success().stdout(count_results_with_header(
        "https://github.com/owner/repo: 1\n",
    ));
    temp_home_directory.close().unwrap();
}
