use crate::branches::count_results_with_header;
use crate::common::test_command;

#[test]
fn returns_empty_results_when_not_passed_any_repository_urls() {
    let mut cmd = test_command();

    let assert = cmd.arg("branches").assert();

    assert.success().stdout(count_results_with_header("\n"));
}
