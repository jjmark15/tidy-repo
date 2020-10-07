use crate::common::test_command;

#[test]
fn errors_when_passed_repository_url_missing_owner() {
    let mut cmd = test_command();

    let assert = cmd.arg("https://github.com//repo").assert();

    assert
        .failure()
        .stderr("Error: failed to parse repository from https://github.com//repo\n");
}
