use crate::branches::branches_command;

#[test]
fn fails_when_passed_repository_url_missing_owner() {
    let assert = branches_command().arg("https://github.com//repo").assert();

    assert
        .failure()
        .stderr("Error: failed to parse repository from 'https://github.com//repo'\n");
}
