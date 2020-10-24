use crate::branches::{branches_command, count_results_with_header};

#[test]
fn returns_empty_results_when_not_passed_any_repository_urls() {
    let temp_home_directory = assert_fs::TempDir::new().unwrap();
    let assert = branches_command(temp_home_directory.path()).assert();

    assert.success().stdout(count_results_with_header("\n"));
    temp_home_directory.close().unwrap();
}
