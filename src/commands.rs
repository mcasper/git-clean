use std::process::{Command, Stdio, Child, Output};
use std::io::{Read, Write};

use options::{GitOptions};

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

pub fn delete_local_branches(branches: &String) -> String {
    let xargs = spawn_piped(vec!["xargs", "git", "branch", "-d"]);

    {
        xargs.stdin.unwrap().write_all(branches.as_bytes()).unwrap()
    }

    let mut s = String::new();
    xargs.stdout.unwrap().read_to_string(&mut s).unwrap();
    s
}

pub fn delete_remote_branches(branches: &String, git_options: &GitOptions) -> String {
    let xargs = spawn_piped(vec!["xargs", "git", "push", &git_options.remote, "--delete"]);

    {
        xargs.stdin.unwrap().write_all(branches.as_bytes()).unwrap()
    }

    let mut stdout = String::new();
    xargs.stdout.unwrap().read_to_string(&mut stdout).unwrap();

    let mut stderr = String::new();
    xargs.stderr.unwrap().read_to_string(&mut stderr).unwrap();

    let split = stderr.split("\n");
    let vec: Vec<&str> = split.collect();
    let mut failed_remotes = vec![];
    for s in vec {
        if s.contains("error: unable to delete '") {
            let branch = s.trim_left_matches("error: unable to delete '")
                .trim_right_matches("': remote ref does not exist");

            failed_remotes.push(branch.to_owned() + " was already deleted in the remote.");
        }
    };
    failed_remotes.join("\n") + &stdout
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
