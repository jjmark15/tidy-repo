use crate::branches::branches_command;

#[test]
fn fails_when_passed_repository_url_missing_repo_name() {
    let temp_home_directory = assert_fs::TempDir::new().unwrap();
    let assert = branches_command(temp_home_directory.path())
        .arg("https://github.com/owner")
        .assert();

    assert
        .failure()
        .stderr("Error: failed to parse repository from 'https://github.com/owner'\n");
    temp_home_directory.close().unwrap();
}
