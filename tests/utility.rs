use support::project;

#[test]
fn test_git_clean_checks_for_git_in_path() {
    let project = project("git-clean_removes").build();

    let result = project.git_clean_command("-y")
        .env("PATH", "")
        .run();

    assert!(!result.is_success(),
            result.failure_message("command to fail"));
    assert!(result.stdout().contains("Unable to execute 'git' on your machine"),
            result.failure_message("to be missing the git command"));
}
