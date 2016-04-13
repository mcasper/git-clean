#[deny(warnings)]

extern crate getopts;

use std::io;
use std::io::{Read, Write};
use std::env;

use getopts::Options;

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
    opts.optopt("R", "", "changes the git remote used (default is origin)", "REMOTE");
    opts.optopt("b", "", "changes the base for merged branches (default is master)", "BRANCH");
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

    let git_options = GitOptions::new(&matches);

    match git_options.validate() {
        Ok(_) => (),
        Err(err) => {
            println!("{}", err);
            return
        }
    }

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
    let base_branch = &git_options.base_branch;
    let regex = format!("(\\*{branch}|\\s{branch})", branch = base_branch);
    let grep = spawn_piped(vec!["grep", "-vE", &regex]);

    let gbranch = run_command(vec!["git", "branch", "--merged", base_branch]);

    {
        grep.stdin.unwrap().write_all(&gbranch.stdout).unwrap();
    }

    let mut s = String::new();
    grep.stdout.unwrap().read_to_string(&mut s).unwrap();

    Branches::new(&s)
}

fn delete_branches(branches: &Branches, options: &DeleteOption, git_options: &GitOptions) -> String {
    match *options {
        DeleteOption::Local => delete_local_branches(branches),
        DeleteOption::Remote => delete_remote_branches(branches, git_options),
        DeleteOption::Both => {
            let out1 = delete_remote_branches(branches, git_options);
            let out2 = delete_local_branches(branches);
            ["Remote:".to_owned(), out1, "\nLocal:".to_owned(), out2].join("\n")
        },
    }
}

#[cfg(test)]
mod test {
    use options::{DeleteOption};
    use branches::Branches;

    use super::print_warning;

    #[test]
    fn test_print_warning() {
        print_warning(&Branches::new(&"branch".to_owned()), &DeleteOption::Both);
    }
}
