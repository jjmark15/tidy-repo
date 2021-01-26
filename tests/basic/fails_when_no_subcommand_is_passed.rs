use crate::common::test_command;

#[test]
fn fails_when_no_subcommand_is_passed() {
    let temp_home_directory = assert_fs::TempDir::new().unwrap();
    let mut cmd = test_command(temp_home_directory.path());

    let assert = cmd.assert();

    assert.failure();
    temp_home_directory.close().unwrap();
}
