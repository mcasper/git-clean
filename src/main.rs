#![deny(warnings)]

extern crate getopts;
extern crate toml;

use std::{env, io};
use std::io::{Read, Write};
use std::error::Error;

mod error;

mod options;
use options::{DeleteMode, Options};

mod branches;
use branches::Branches;

mod commands;
use commands::*;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut opts = getopts::Options::new();
    opts.optflag("l", "locals", "only delete local branches");
    opts.optflag("r", "remotes", "only delete remote branches");
    opts.optflag("y", "yes", "skip the check for deleting branches");
    opts.optflag("s",
                 "squashes",
                 "check for squashes by finding branches incompatible with master");
    opts.optopt("R",
                "remote",
                "changes the git remote used (default is origin)",
                "REMOTE");
    opts.optopt("b",
                "branch",
                "changes the base for merged branches (default is master)",
                "BRANCH");
    opts.optmulti("i", "ignore", "ignores given branches", "BRANCH");
    opts.optflag("h", "help", "print this help menu");
    opts.optflag("v", "version", "print the version");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(e) => {
            println!("{}", e);
            print_help(&opts);
            return;
        }
    };

    if matches.opt_present("h") {
        print_help(&opts);
        return;
    }

    if matches.opt_present("version") {
        print_version();
        return;
    }

    validate_git_installation().unwrap_or_else(|e| print_and_exit(&e));

    let options = Options::new(&matches);
    options.validate().unwrap_or_else(|e| print_and_exit(&e));

    let branches = merged_branches(&options);

    if branches.string.is_empty() {
        println!("No branches to delete, you're clean!");
        return;
    }

    if !matches.opt_present("y") {
        print_warning(&branches, &options.delete_mode);

        // Read the user's response on continuing
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        match input.to_lowercase().as_ref() {
            "y\n" | "yes\n" | "\n" => (),
            _ => return,
        }
    }

    let msg = delete_branches(&branches, &options);
    println!("\n{}", msg);
}

fn print_version() {
    println!("git-clean version {}", VERSION);
}

fn print_help(opts: &getopts::Options) {
    print!("{}", opts.usage("Usage: git-clean [options]"));
}

fn print_warning(branches: &Branches, delete_mode: &DeleteMode) {
    println!("{}", delete_mode.warning_message());
    println!("{}", branches.format_columns());
    print!("Continue? (Y/n) ");
    io::stdout().flush().unwrap();
}

fn merged_branches(options: &Options) -> Branches {
    let mut branches: Vec<String> = vec![];
    println!("Updating remote {}", options.remote);
    run_command_with_no_output(&["git", "remote", "update", &options.remote, "--prune"]);

    let merged_branches_regex = format!("\\*{branch}|\\s{branch}",
                                        branch = &options.base_branch);
    let merged_branches_filter = spawn_piped(&["grep", "-vE", &merged_branches_regex]);
    let merged_branches_cmd = run_command(&["git", "branch", "--merged"]);

    {
        merged_branches_filter.stdin.unwrap().write_all(&merged_branches_cmd.stdout).unwrap();
    }

    let mut merged_branches_output = String::new();
    merged_branches_filter.stdout.unwrap().read_to_string(&mut merged_branches_output).unwrap();
    let merged_branches =
        merged_branches_output.split('\n').map(|b| b.trim().into()).collect::<Vec<String>>();

    let local_branches_regex = format!("\\*{branch}|\\s{branch}",
                                       branch = &options.base_branch);
    let local_branches_filter = spawn_piped(&["grep", "-vE", &local_branches_regex]);
    let local_branches_cmd = run_command(&["git", "branch"]);

    {
        local_branches_filter.stdin.unwrap().write_all(&local_branches_cmd.stdout).unwrap();
    }

    let mut local_branches_output = String::new();
    local_branches_filter.stdout.unwrap().read_to_string(&mut local_branches_output).unwrap();

    let local_branches = local_branches_output.split('\n')
        .map(|b| b.trim().into())
        .filter(|branch| !options.ignored_branches.contains(branch))
        .collect::<Vec<String>>();

    let remote_branches_regex = format!("(HEAD|{})", &options.base_branch);
    let remote_branches_filter = spawn_piped(&["grep", "-vE", &remote_branches_regex]);
    let remote_branches_cmd = run_command(&["git", "branch", "-r"]);

    {
        remote_branches_filter.stdin.unwrap().write_all(&remote_branches_cmd.stdout).unwrap();
    }

    let mut remote_branches_output = String::new();
    remote_branches_filter.stdout.unwrap().read_to_string(&mut remote_branches_output).unwrap();
    let remote_branches =
        remote_branches_output.split('\n').map(|b| b.trim().into()).collect::<Vec<String>>();

    for branch in local_branches {
        // First check if the local branch doesn't exist in the remote, it's the cheapest and easiest
        // way to determine if we want to suggest to delete it.
        if !remote_branches.contains(&format!("{}/{}", &options.remote, branch)) {
            branches.push(branch.to_owned());
            continue;
        }

        // If it does exist in the remote, check to see if it's listed in git branches --merged. If
        // it is, that means it wasn't merged using Github squashes, and we can suggest it.
        if merged_branches.contains(&branch) {
            branches.push(branch.to_owned());
            continue;
        }

        // If neither of the above matched, merge master into the branch and see if it succeeds.
        // If it can't cleanly merge, then it has likely been merged with Github squashes, and we
        // can suggest it.
        if options.squashes {
            run_command(&["git", "checkout", &branch]);
            match run_command_with_status(&["git",
                                            "pull",
                                            "--ff-only",
                                            &options.remote,
                                            &options.base_branch]) {
                Ok(status) => {
                    if !status.success() {
                        println!("why");
                        branches.push(branch.into());
                    }
                }
                Err(err) => {
                    println!("Encountered error trying to update branch {} with branch {}: {}",
                             branch,
                             options.base_branch,
                             err);
                    continue;
                }
            }

            run_command(&["git", "reset", "--hard"]);
            run_command(&["git", "checkout", &options.base_branch]);
        }
    }

    // if deleted in remote, list
    //
    // g branch -d -r <remote>/<branch>
    // g branch -d <branch>

    Branches::new(branches)
}

fn delete_branches(branches: &Branches,
                   options: &Options)
                   -> String {
    match options.delete_mode {
        DeleteMode::Local => delete_local_branches(branches),
        DeleteMode::Remote => delete_remote_branches(branches, options),
        DeleteMode::Both => {
            let local_output = delete_local_branches(branches);
            let remote_output = delete_remote_branches(branches, options);
            ["Remote:".to_owned(), remote_output, "\nLocal:".to_owned(), local_output].join("\n")
        }
    }
}

fn print_and_exit<E: Error>(e: &E) {
    println!("{}", e);
    std::process::exit(1);
}

#[cfg(test)]
mod test {
    use options::DeleteMode;
    use branches::Branches;

    use super::print_warning;

    #[test]
    fn test_print_warning() {
        print_warning(&Branches::new(vec!["branch".to_owned()]),
                      &DeleteMode::Both);
    }
}
