use crate::common::test_command;

#[test]
fn fails_when_passed_repository_url_missing_repo_name() {
    let mut cmd = test_command();

    let assert = cmd.arg("https://github.com/owner").assert();

    assert
        .failure()
        .stderr("Error: failed to parse repository from https://github.com/owner\n");
}
