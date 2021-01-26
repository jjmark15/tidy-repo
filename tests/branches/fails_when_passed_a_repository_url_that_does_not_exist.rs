use crate::branches::branches_command;
use crate::common::mock_github_api_server_for_repository_not_found;

#[test]
fn fails_when_passed_repository_url_that_does_not_exist() {
    let temp_home_directory = assert_fs::TempDir::new().unwrap();
    let body_string = "[{\"message\": \"Not Found\"}]";
    let _mock =
        mock_github_api_server_for_repository_not_found("owner", "repo", body_string).create();

    let assert = branches_command(temp_home_directory.path())
        .arg("https://github.com/owner/repo")
        .assert();

    assert
        .failure()
        .stderr("Error: repository 'https://github.com/owner/repo' not found\n");
    temp_home_directory.close().unwrap();
}
