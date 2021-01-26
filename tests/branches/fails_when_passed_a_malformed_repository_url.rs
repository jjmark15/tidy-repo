use crate::branches::branches_command;

#[test]
fn fails_when_passed_a_malformed_repository_url() {
    let temp_home_directory = assert_fs::TempDir::new().unwrap();
    let assert = branches_command(temp_home_directory.path())
        .arg("https://not-github.com/owner/repo")
        .assert();

    assert
        .failure()
        .stderr("Error: failed to parse repository from 'https://not-github.com/owner/repo'\n");
    temp_home_directory.close().unwrap();
}
