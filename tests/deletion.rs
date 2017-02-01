use support::project;

#[test]
fn test_git_clean_works_with_merged_branches() {
    let project = project("git-clean_squashed_merges").build().setup_remote();

    project.batch_setup_commands(
        &[
            "git checkout -b merged",
            "touch file2.txt",
            "git add .",
            "git commit -am Merged",
            "git checkout master",
            "git merge merged",
        ]
    );

    let result = project.git_clean_command("-y").run();

    assert!(result.is_success(), result.failure_message("command to succeed"));
    assert!(result.stdout().contains("Deleted branch merged"), result.failure_message("command to delete merged"));
}

#[test]
fn test_git_clean_works_with_squashed_merges() {
    let project = project("git-clean_squashed_merges").build().setup_remote();

    project.batch_setup_commands(
        &[
            "git checkout -b squashed",
            "touch file2.txt",
            "git add .",
            "git commit -am Squash",
            "git checkout master",
            "git merge --ff-only squashed",
        ]
    );

    let result = project.git_clean_command("-y").run();

    assert!(result.is_success(), result.failure_message("command to succeed"));
    assert!(result.stdout().contains("Deleted branch squashed"), result.failure_message("command to delete squashed"));
}

#[test]
fn test_git_clean_does_not_delete_branches_ahead_of_master() {
    let project = project("git-clean_branch_ahead").build().setup_remote();

    project.batch_setup_commands(
        &[
            "git checkout -b ahead",
            "touch file2.txt",
            "git add .",
            "git commit -am Ahead",
            "git push origin HEAD",
            "git checkout master",
        ]
    );

    let result = project.git_clean_command("-y").run();

    assert!(result.is_success(), result.failure_message("command to succeed"));
    assert!(!result.stdout().contains("Deleted branch ahead"), result.failure_message("command not to delete ahead"));
}

#[test]
fn test_git_clean_works_with_squashes_with_flag() {
    let project = project("git-clean_github_squashes").build().setup_remote();

    // Github squashes function basically like a normal squashed merge, it creates an entirely new
    // commit in which all your changes live. The biggest challenge of this is that your local
    // branch doesn't have any knowledge of this new commit. So if master gets ahead of your local
    // branch, git no longer is able to tell that branch has been merged. These commands simulate
    // this condition.
    project.batch_setup_commands(
        &[
            "git checkout -b github_squash",
            "touch squash.txt",
            "git add .",
            "git commit -am Commit",
            "git push origin HEAD",
            "git checkout master",
            "touch squash.txt",
            "git add .",
            "git commit -am Squash",
            "touch new.txt",
            "git add .",
            "git commit -am Other",
            "git push origin HEAD"
        ]
    );

    let result = project.git_clean_command("-y --squashes").run();

    assert!(result.is_success(), result.failure_message("command to succeed"));
    assert!(result.stdout().contains(" - [deleted]         github_squash"), result.failure_message("command to delete github_squash in remote"));
    assert!(result.stdout().contains("Deleted branch github_squash"), result.failure_message("command to delete github_squash locally"));
}

#[test]
fn test_git_clean_ignores_squashes_without_flag() {
    let project = project("git-clean_ignores_github_squashes").build().setup_remote();

    // Github squashes function basically like a normal squashed merge, it creates an entirely new
    // commit in which all your changes live. The biggest challenge of this is that your local
    // branch doesn't have any knowledge of this new commit. So if master gets ahead of your local
    // branch, git no longer is able to tell that branch has been merged. These commands simulate
    // this condition.
    project.batch_setup_commands(
        &[
            "git checkout -b github_squash",
            "touch squash.txt",
            "git add .",
            "git commit -am Commit",
            "git push origin HEAD",
            "git checkout master",
            "touch squash.txt",
            "git add .",
            "git commit -am Squash",
            "touch new.txt",
            "git add .",
            "git commit -am Other",
            "git push origin HEAD"
        ]
    );

    let result = project.git_clean_command("-y").run();

    assert!(result.is_success(), result.failure_message("command to succeed"));
    assert!(!result.stdout().contains(" - [deleted]         github_squash"), result.failure_message("command not to delete github_squash in remote"));
    assert!(!result.stdout().contains("Deleted branch github_squash"), result.failure_message("command not to delete github_squash locally"));
}
