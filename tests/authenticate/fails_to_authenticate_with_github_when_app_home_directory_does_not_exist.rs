use assert_fs::fixture::PathChild;
use predicates::str::ends_with;

use crate::authenticate::{
    authenticate_command, mock_github_api_server_for_successful_authentication_check,
};

#[test]
fn fails_to_authenticate_with_github_when_app_home_directory_does_not_exist() {
    let temp_home_directory = assert_fs::TempDir::new().unwrap();
    temp_home_directory.child("inner");
    let _mock = mock_github_api_server_for_successful_authentication_check("OAUTH-TOKEN").create();
    let assert = authenticate_command(temp_home_directory.child("inner").path())
        .arg("github")
        .arg("--token")
        .arg("OAUTH-TOKEN")
        .assert();

    assert
        .failure()
        .stderr(ends_with("Failed to store credential\n"));
    temp_home_directory.close().unwrap();
}
