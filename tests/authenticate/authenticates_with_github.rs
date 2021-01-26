use crate::authenticate::{
    authenticate_command, mock_github_api_server_for_successful_authentication_check,
};
use crate::common::GITHUB_OAUTH_TOKEN;

#[test]
fn authenticate_with_github_inline() {
    let temp_home_directory = assert_fs::TempDir::new().unwrap();
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
