use predicates::str::ends_with;

use crate::authenticate::{
    authenticate_command, mock_github_api_server_for_unsuccessful_authentication_check,
};

#[test]
fn fails_to_authenticate_with_github_when_passed_fake_token() {
    let temp_home_directory = assert_fs::TempDir::new().unwrap();
    let _mock =
        mock_github_api_server_for_unsuccessful_authentication_check("OAUTH-TOKEN").create();
    let assert = authenticate_command(temp_home_directory.path())
        .arg("github")
        .arg("--token")
        .arg("OAUTH-TOKEN")
        .assert();

    assert.failure().stderr(ends_with("invalid credentials\n"));
    temp_home_directory.close().unwrap();
}

#[test]
fn fails_to_authenticate_with_github_when_passed_empty_token() {
    let temp_home_directory = assert_fs::TempDir::new().unwrap();
    let assert = authenticate_command(temp_home_directory.path())
        .arg("github")
        .arg("--token")
        .arg("")
        .assert();

    assert
        .failure()
        .stderr(ends_with("GitHub authentication token must not be empty\n"));
    temp_home_directory.close().unwrap();
}
