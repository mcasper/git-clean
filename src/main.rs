#![deny(warnings)]

extern crate getopts;

use std::{env, io};
use std::io::{Read, Write};
use std::error::Error;

use getopts::Options;

mod error;

mod options;
use options::{DeleteOption, GitOptions};

mod branches;
use branches::Branches;

mod commands;
use commands::*;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut opts = Options::new();
    opts.optflag("l", "locals", "only delete local branches");
    opts.optflag("r", "remotes", "only delete remote branches");
    opts.optflag("y", "yes", "skip the check for deleting branches");
    opts.optopt("R", "remote", "changes the git remote used (default is origin)", "REMOTE");
    opts.optopt("b", "branch", "changes the base for merged branches (default is master)", "BRANCH");
    opts.optflag("h", "help", "print this help menu");
    opts.optflag("v", "version", "print the version");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(e) => {
            println!("{}", e);
            print_help(opts);
            return;
        }
    };

    if matches.opt_present("h") {
        print_help(opts);
        return;
    }

    if matches.opt_present("version") {
        print_version();
        return;
    }

    validate_git_installation().unwrap_or_else(print_and_exit);

    let git_options = GitOptions::new(&matches);
    git_options.validate().unwrap_or_else(print_and_exit);

    let branches = merged_branches(&git_options);

    if branches.string.len() == 0 {
        println!("No branches to delete, you're clean!");
        return;
    }

    let del_opt = DeleteOption::new(&matches);

    if !matches.opt_present("y") {
        print_warning(&branches, &del_opt);

        // Read the user's response on continuing
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        match input.to_lowercase().as_ref() {
            "y\n" => (),
            "yes\n" => (),
            "\n" => (),
            _ => return,
        }
    }

    let msg = delete_branches(&branches, &del_opt, &git_options);
    println!("\n{}", msg);
}

fn print_version() {
    println!("git-clean version {}", VERSION);
}

fn print_help(opts: Options) {
    print!("{}", opts.usage("Usage: git-clean [options]"));
}

fn print_warning(branches: &Branches, del_opt: &DeleteOption) {
    println!("{}", del_opt.warning_message());
    println!("{}", branches.format_columns());
    print!("Continue? (Y/n) ");
    io::stdout().flush().unwrap();
}

fn merged_branches(git_options: &GitOptions) -> Branches {
    let mut branches: Vec<String> = vec![];
    println!("Updating remote {}", git_options.remote);
    run_command_with_no_output(&["git", "remote", "update", &git_options.remote, "--prune"]);

    let merged_branches_regex = format!("\\*{branch}|\\s{branch}", branch = &git_options.base_branch);
    let merged_branches_filter = spawn_piped(&["grep", "-vE", &merged_branches_regex]);
    let merged_branches_cmd = run_command(&["git", "branch", "--merged"]);

    {
        merged_branches_filter.stdin.unwrap().write_all(&merged_branches_cmd.stdout).unwrap();
    }

    let mut merged_branches_output = String::new();
    merged_branches_filter.stdout.unwrap().read_to_string(&mut merged_branches_output).unwrap();
    let merged_branches = merged_branches_output.split("\n").map(|b| b.trim().into()).collect::<Vec<String>>();

    let local_branches_regex = format!("\\*{branch}|\\s{branch}", branch = &git_options.base_branch);
    let local_branches_filter = spawn_piped(&["grep", "-vE", &local_branches_regex]);
    let local_branches_cmd = run_command(&["git", "branch"]);

    {
        local_branches_filter.stdin.unwrap().write_all(&local_branches_cmd.stdout).unwrap();
    }

    let mut local_branches_output = String::new();
    local_branches_filter.stdout.unwrap().read_to_string(&mut local_branches_output).unwrap();
    let local_branches = local_branches_output.split("\n").map(|b| b.trim().into()).collect::<Vec<String>>();

    let remote_branches_regex = format!("(HEAD|{})", &git_options.base_branch);
    let remote_branches_filter = spawn_piped(&["grep", "-vE", &remote_branches_regex]);
    let remote_branches_cmd = run_command(&["git", "branch", "-r"]);

    {
        remote_branches_filter.stdin.unwrap().write_all(&remote_branches_cmd.stdout).unwrap();
    }

    let mut remote_branches_output = String::new();
    remote_branches_filter.stdout.unwrap().read_to_string(&mut remote_branches_output).unwrap();
    let remote_branches = remote_branches_output.split("\n").map(|b| b.trim().into()).collect::<Vec<String>>();

    for branch in local_branches {
        // First check if the local branch doesn't exist in the remote, it's the cheapest and easiest
        // way to determine if we want to suggest to delete it.
        if !remote_branches.contains(&format!("{}/{}", &git_options.remote, branch)) {
            branches.push(branch.to_owned());
            continue;
        }

        // If it does exist in the remote, check to see if it's listed in git branches --merged. If
        // it is, that means it wasn't merged using Github squashes, and we can suggest it.
        if merged_branches.contains(&branch) {
            branches.push(branch.to_owned());
            continue;
        }

        // If neither of the above matched, merge master into the branch and see if there's any
        // diff. If there's no diff, then it has likely been merged with Github squashes, and we
        // can suggest it.
        run_command(&["git", "checkout", &branch]);
        match run_command_with_status(&["git", "pull", &git_options.remote, &git_options.base_branch]) {
            Ok(status) => {
                if !status.success() {
                    println!("Encountered error trying to update branch {}, skipping", branch);
                    run_command(&["git", "reset", "--hard"]);
                    continue;
                }
            }
            Err(err) => {
                println!("Encountered error trying to update branch {} with branch {}: {}", branch, git_options.base_branch, err);
                run_command(&["git", "reset", "--hard"]);
                continue;
            }
        }
        let git_diff_cmd = run_command(&["git", "diff", &git_options.base_branch]);
        let git_diff = String::from_utf8(git_diff_cmd.stdout).unwrap();

        // If there's no diff with the base branch, suggest it.
        if git_diff.trim().len() == 0 {
            branches.push(branch.to_owned());
        }

        run_command(&["git", "checkout", &git_options.base_branch]);
    }

    // if deleted in remote, list
    //
    // g branch -d -r <remote>/<branch>
    // g branch -d <branch>

    Branches::new(branches)
}

fn delete_branches(branches: &Branches, options: &DeleteOption, git_options: &GitOptions) -> String {
    match *options {
        DeleteOption::Local => delete_local_branches(branches),
        DeleteOption::Remote => delete_remote_branches(branches, git_options),
        DeleteOption::Both => {
            let local_output = delete_local_branches(branches);
            let remote_output = delete_remote_branches(branches, git_options);
            ["Remote:".to_owned(), remote_output, "\nLocal:".to_owned(), local_output].join("\n")
        },
    }
}

fn print_and_exit<E: Error>(e: E) {
    println!("{}", e);
    std::process::exit(1);
}

#[cfg(test)]
mod test {
    use options::{DeleteOption};
    use branches::Branches;

    use super::print_warning;

    #[test]
    fn test_print_warning() {
        print_warning(&Branches::new(vec!["branch".to_owned()]), &DeleteOption::Both);
    }
}
