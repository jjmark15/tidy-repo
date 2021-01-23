use predicates::str::ends_with;

use crate::authenticate::{
    authenticate_command, mock_github_api_server_for_successful_authentication_check,
};
use crate::common::APP_HOME_ENVIRONMENT_VARIABLE;

#[test]
fn fails_when_app_home_environment_variable_is_not_set() {
    let temp_home_directory = assert_fs::TempDir::new().unwrap();
    let _mock = mock_github_api_server_for_successful_authentication_check("OAUTH-TOKEN").create();
    let assert = authenticate_command(temp_home_directory.path())
        .env_remove(APP_HOME_ENVIRONMENT_VARIABLE)
        .arg("github")
        .arg("--token")
        .arg("OAUTH-TOKEN")
        .assert();

    assert.failure().stderr(ends_with(
        "TIDY_REPO_HOME environment variable is not set\n",
    ));
    temp_home_directory.close().unwrap();
}
