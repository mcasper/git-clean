use std::process::{Command, Stdio, Child, Output};
use std::io::{Read, Write};
use std::collections::BTreeSet;

use options::{GitOptions};
use branches::Branches;

pub fn spawn_piped(args: Vec<&str>) -> Child {
    let cmd = args[0];
    Command::new(cmd)
        .args(&args[1..])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| { panic!("Error with child process: {}", e) })
}

pub fn run_command(args: Vec<&str>) -> Output {
    let cmd = args[0];
    Command::new(cmd)
        .args(&args[1..])
        .output()
        .unwrap_or_else(|e| { panic!("Error with command: {}", e) })
}

pub fn delete_local_branches(branches: &Branches) -> String {
    let xargs = spawn_piped(vec!["xargs", "git", "branch", "-d"]);

    {
        xargs.stdin.unwrap().write_all(&branches.string.as_bytes()).unwrap()
    }

    let mut s = String::new();
    xargs.stdout.unwrap().read_to_string(&mut s).unwrap();
    s
}

pub fn delete_remote_branches(branches: &Branches, git_options: &GitOptions) -> String {
    let xargs = spawn_piped(vec!["xargs", "git", "push", &git_options.remote, "--delete"]);

    let remote_branches_cmd = run_command(vec!["git", "branch", "-r"]);

    let s = String::from_utf8(remote_branches_cmd.stdout).unwrap();
    let split = s.split("\n");
    let all_remote_branches = split.collect::<Vec<&str>>();
    let b_tree_remotes = all_remote_branches
        .iter()
        .map(|b| b.trim().trim_left_matches("origin/").to_owned())
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
    let split = stderr.split("\n");
    let vec: Vec<&str> = split.collect();
    let mut output = vec![];
    for s in vec {
        if s.contains("error: unable to delete '") {
            let branch = s.trim_left_matches("error: unable to delete '")
                .trim_right_matches("': remote ref does not exist");

            output.push(branch.to_owned() + " was already deleted in the remote or was never there.");
        } else if s.contains(" - [deleted]") {
            output.push(s.to_owned());
        }
    };

    output.join("\n")
}

#[cfg(test)]
mod test {
    use super::spawn_piped;

    use std::io::{Read, Write};

    #[test]
    fn test_spawn_piped() {
        let echo = spawn_piped(vec!["grep", "foo"]);

        {
            echo.stdin.unwrap().write_all("foo\nbar\nbaz".as_bytes()).unwrap()
        }

        let mut stdout = String::new();
        echo.stdout.unwrap().read_to_string(&mut stdout).unwrap();

        assert_eq!(stdout, "foo\n");
    }
}
