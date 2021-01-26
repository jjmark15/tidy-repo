use predicates::str::ends_with;

use crate::branches::{branches_command, mock_github_api_server_for_successful_list_branches};
use crate::common::APP_HOME_ENVIRONMENT_VARIABLE;

#[test]
fn fails_when_app_home_environment_variable_is_not_set() {
    let temp_home_directory = assert_fs::TempDir::new().unwrap();
    let body_string = "[{\"name\": \"branch\"}]";
    let _mock =
        mock_github_api_server_for_successful_list_branches("owner", "repo", body_string).create();

    let assert = branches_command(temp_home_directory.path())
        .arg("https://github.com/owner/repo")
        .env_remove(APP_HOME_ENVIRONMENT_VARIABLE)
        .assert();

    assert.failure().stderr(ends_with(
        "TIDY_REPO_HOME environment variable is not set\n",
    ));
    temp_home_directory.close().unwrap();
}
