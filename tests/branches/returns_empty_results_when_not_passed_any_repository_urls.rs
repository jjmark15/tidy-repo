use crate::branches::{branches_command, count_results_with_header};

#[test]
fn returns_empty_results_when_not_passed_any_repository_urls() {
    let assert = branches_command().assert();

    assert.success().stdout(count_results_with_header("\n"));
}
