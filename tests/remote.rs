use support::project;

#[test]
fn test_git_clean_removes_remote_branches() {
    let project = project("git-clean_removes_remote").build().setup_remote();

    project.batch_setup_commands(
        &[
            "git checkout -b test1",
            "git push origin HEAD",
            "git checkout -b test2",
            "git push origin HEAD",
            "git checkout master",
        ]
    );

    let verify = project.setup_command("git branch -r");

    assert!(verify.stdout().contains("test1"), verify.failure_message("test1 to exist in remote"));
    assert!(verify.stdout().contains("test2"), verify.failure_message("test2 to exist in remote"));

    let result = project.git_clean_command("-y").run();

    assert!(result.is_success(), result.failure_message("command to succeed"));
    assert!(result.stdout().contains(deleted_branch_output("test1").as_str()), result.failure_message("command to delete test1"));
    assert!(result.stdout().contains(deleted_branch_output("test2").as_str()), result.failure_message("command to delete test2"));
}

#[test]
fn test_git_clean_does_not_remove_ignored_remote_branches() {
    let project = project("git-clean_removes_remote").build().setup_remote();

    project.batch_setup_commands(
        &[
            "git checkout -b test1",
            "git push origin HEAD",
            "git checkout -b test2",
            "git push origin HEAD",
            "git checkout master",
        ]
    );

    let verify = project.setup_command("git branch -r");

    assert!(verify.stdout().contains("test1"), verify.failure_message("test1 to exist in remote"));
    assert!(verify.stdout().contains("test2"), verify.failure_message("test2 to exist in remote"));

    let result = project.git_clean_command("-y -i test2").run();

    assert!(result.is_success(), result.failure_message("command to succeed"));
    assert!(result.stdout().contains(deleted_branch_output("test1").as_str()), result.failure_message("command to delete test1"));
    assert!(!result.stdout().contains(deleted_branch_output("test2").as_str()), result.failure_message("command to delete test2"));
}

fn deleted_branch_output(branch: &str) -> String {
    format!(" - [deleted]         {}", branch)
}
