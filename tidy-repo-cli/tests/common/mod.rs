use assert_cmd::Command;

pub(crate) fn test_command() -> Command {
    let mut cmd = Command::cargo_bin("tidy-repo").expect("Could not run cargo binary 'tidy-repo'");
    cmd.env("TIDY_REPO_GITHUB_API_BASE_URL", mockito::server_url());
    cmd
}
