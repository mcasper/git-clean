use support::project;

#[test]
fn test_git_clean_removes_local_branches() {
    let project = project("git-clean_removes").build();

    project.setup_command("git branch test1");
    project.setup_command("git branch test2");

    let verify = project.setup_command("git branch");

    assert!(verify.stdout().contains("test1"), verify.failure_message("test1"));
    assert!(verify.stdout().contains("test2"), verify.failure_message("test2"));

    let result = project.git_clean_command("-y");

    assert!(result.is_success(), result.failure_message("command to succeed"));
    assert!(result.stdout().contains("Deleted branch test1"), result.failure_message("command to delete test1"));
    assert!(result.stdout().contains("Deleted branch test2"), result.failure_message("command to delete test2"));
}

#[test]
fn test_git_clean_works_with_squashed_merges() {
    let project = project("git-clean_squashed_merges").build();

    project.batch_setup_commands(
        vec![
            "git checkout -b squashed",
            "touch file2.txt",
            "git add .",
            "git commit -am Squash",
            "git checkout master",
            "git merge --ff-only squashed",
        ]
    );

    let result = project.git_clean_command("-y");

    assert!(result.is_success(), result.failure_message("command to succeed"));
    assert!(result.stdout().contains("Deleted branch squashed"), result.failure_message("command to delete squashed"));
}

#[test]
fn test_git_clean_does_not_delete_branches_ahead_of_master() {
    let project = project("git-clean_branch_ahead").build();

    project.batch_setup_commands(
        vec![
            "git checkout -b ahead",
            "touch file2.txt",
            "git add .",
            "git commit -am Ahead",
            "git checkout master",
        ]
    );

    let result = project.git_clean_command("-y");

    assert!(result.is_success(), result.failure_message("command to succeed"));
    assert!(!result.stdout().contains("Deleted branch ahead"), result.failure_message("command not to delete ahead"));
}
