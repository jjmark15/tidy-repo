use crate::common::test_command;

#[test]
fn fails_when_passed_a_malformed_repository_url() {
    let mut cmd = test_command();

    let assert = cmd.arg("https://not-github.com/owner/repo").assert();

    assert
        .failure()
        .stderr("Error: failed to parse repository from 'https://not-github.com/owner/repo'\n");
}
