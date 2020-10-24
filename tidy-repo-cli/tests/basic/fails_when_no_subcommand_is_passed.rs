use crate::common::test_command;

#[test]
fn fails_when_no_subcommand_is_passed() {
    let mut cmd = test_command();

    let assert = cmd.assert();

    assert.failure();
}
