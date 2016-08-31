use support::project;

#[test]
fn test_git_clean_removes_local_branches() {
    let project = project("git-clean_removes_local").build();

    project.setup_command("git branch test1");
    project.setup_command("git branch test2");

    let verify = project.setup_command("git branch");

    assert!(verify.stdout().contains("test1"), verify.failure_message("test1"));
    assert!(verify.stdout().contains("test2"), verify.failure_message("test2"));

    let result = project.git_clean_command("-y").run();

    assert!(result.is_success(), result.failure_message("command to succeed"));
    assert!(result.stdout().contains("Deleted branch test1"), result.failure_message("command to delete test1"));
    assert!(result.stdout().contains("Deleted branch test2"), result.failure_message("command to delete test2"));
}
