#![deny(warnings)]

extern crate clap;

use std::io::{Read, Write, stdin, stdout};
use std::error::Error;

mod app;

mod error;

mod options;
use options::{DeleteMode, Options};

mod branches;
use branches::Branches;

mod commands;
use commands::*;

fn main() {
    let matches = app::app().get_matches();

    validate_git_installation().unwrap_or_else(|e| print_and_exit(&e));

    let options = Options::new(&matches);
    options.validate().unwrap_or_else(|e| print_and_exit(&e));

    let branches = merged_branches(&options);

    if branches.string.is_empty() {
        println!("No branches to delete, you're clean!");
        return;
    }

    if !matches.is_present("yes") {
        print_warning(&branches, &options.delete_mode);

        // Read the user's response on continuing
        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();

        match input.to_lowercase().as_ref() {
            "y\n" | "yes\n" | "\n" => (),
            _ => return,
        }
    }

    let msg = delete_branches(&branches, &options);
    println!("\n{}", msg);
}

fn print_warning(branches: &Branches, delete_mode: &DeleteMode) {
    println!("{}", delete_mode.warning_message());
    println!("{}", branches.format_columns());
    print!("Continue? (Y/n) ");
    stdout().flush().unwrap();
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
