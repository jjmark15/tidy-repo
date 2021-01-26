use crate::authenticate::{
    authenticate_command, mock_github_api_server_for_successful_authentication_check,
};
use crate::common::{authenticate_session_with_github_with_token, GITHUB_OAUTH_TOKEN};

#[test]
fn authentication_with_github_overwrites_previous_github_authentication() {
    let temp_home_directory = assert_fs::TempDir::new().unwrap();
    let previous_token = "previous token";
    authenticate_session_with_github_with_token(temp_home_directory.path(), previous_token)
        .unwrap();
    let _mock =
        mock_github_api_server_for_successful_authentication_check(GITHUB_OAUTH_TOKEN).create();
    let assert = authenticate_command(temp_home_directory.path())
        .arg("github")
        .arg("--token")
        .arg(GITHUB_OAUTH_TOKEN)
        .assert();

    assert
        .success()
        .stdout("Successfully authenticated with GitHub\n");
    temp_home_directory.close().unwrap();
}
