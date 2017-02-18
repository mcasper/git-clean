use std::process::{Command, Child, ExitStatus, Output, Stdio};
use std::io::{Read, Write, Error as IOError};
use std::collections::BTreeSet;

use branches::Branches;
use error::GitCleanError;
use options::GitOptions;

pub fn spawn_piped(args: &[&str]) -> Child {
    Command::new(&args[0])
        .args(&args[1..])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| panic!("Error with child process: {}", e))
}

pub fn run_command_with_no_output(args: &[&str]) {
    Command::new(&args[0])
        .args(&args[1..])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .unwrap_or_else(|e| panic!("Error with command: {}", e));
}

pub fn output(args: &[&str]) -> String {
    let result = run_command(args);
    String::from_utf8(result.stdout).unwrap().trim().to_owned()
}

pub fn run_command(args: &[&str]) -> Output {
    run_command_with_result(args).unwrap_or_else(|e| panic!("Error with command: {}", e))
}

pub fn run_command_with_result(args: &[&str]) -> Result<Output, IOError> {
    Command::new(&args[0])
        .args(&args[1..])
        .output()
}

pub fn run_command_with_status(args: &[&str]) -> Result<ExitStatus, IOError> {
    Command::new(&args[0])
        .args(&args[1..])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
}

pub fn validate_git_installation() -> Result<(), GitCleanError> {
    match Command::new("git").output() {
        Ok(_) => Ok(()),
        Err(_) => Err(GitCleanError::GitInstallationError),
    }
}

pub fn delete_local_branches(branches: &Branches) -> String {
    let xargs = spawn_piped(&["xargs", "git", "branch", "-D"]);

    {
        xargs.stdin.unwrap().write_all(branches.string.as_bytes()).unwrap()
    }

    let mut branches_delete_result = String::new();
    xargs.stdout.unwrap().read_to_string(&mut branches_delete_result).unwrap();
    branches_delete_result
}

pub fn delete_remote_branches(branches: &Branches, git_options: &GitOptions) -> String {
    let xargs = spawn_piped(&["xargs", "git", "push", &git_options.remote, "--delete"]);

    let remote_branches_cmd = run_command(&["git", "branch", "-r"]);

    let s = String::from_utf8(remote_branches_cmd.stdout).unwrap();
    let all_remote_branches = s.split('\n').collect::<Vec<&str>>();
    let origin_for_trim = &format!("{}/", &git_options.remote)[..];
    let b_tree_remotes = all_remote_branches.iter()
        .map(|b| b.trim().trim_left_matches(origin_for_trim).to_owned())
        .collect::<BTreeSet<String>>();

    let mut b_tree_branches = BTreeSet::new();

    for branch in branches.vec.clone() {
        b_tree_branches.insert(branch);
    }

    let intersection: Vec<_> = b_tree_remotes.intersection(&b_tree_branches).cloned().collect();

    {
        xargs.stdin.unwrap().write_all(intersection.join("\n").as_bytes()).unwrap()
    }

    let mut stderr = String::new();
    xargs.stderr.unwrap().read_to_string(&mut stderr).unwrap();

    // Everything is written to stderr, so we need to process that
    let split = stderr.split('\n');
    let vec: Vec<&str> = split.collect();
    let mut output = vec![];
    for s in vec {
        if s.contains("error: unable to delete '") {
            let branch = s.trim_left_matches("error: unable to delete '")
                .trim_right_matches("': remote ref does not exist");

            output.push(branch.to_owned() + " was already deleted in the remote.");
        } else if s.contains(" - [deleted]") {
            output.push(s.to_owned());
        }
    }

    output.join("\n")
}

#[cfg(test)]
mod test {
    use super::spawn_piped;

    use std::io::{Read, Write};

    #[test]
    fn test_spawn_piped() {
        let echo = spawn_piped(&["grep", "foo"]);

        {
            echo.stdin.unwrap().write_all("foo\nbar\nbaz".as_bytes()).unwrap()
        }

        let mut stdout = String::new();
        echo.stdout.unwrap().read_to_string(&mut stdout).unwrap();

        assert_eq!(stdout, "foo\n");
    }
}
