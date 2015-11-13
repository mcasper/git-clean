extern crate getopts;

use std::process::{Command, Stdio};
use std::io;
use std::io::{Read, Write};
use std::env;

use getopts::Options;

enum DeleteOption {
    Local,
    Remote,
    Both
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut opts = Options::new();
    opts.optflag("l", "local", "only delete local branches");
    opts.optflag("r", "remote", "only delete remote branches");
    opts.optflag("h", "help", "print this help menu");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };

    if matches.opt_present("h") {
        print_usage(opts);
        return;
    }

    let branches = merged_branches();

    // Early return if there's nothing to delete
    if branches.len() == 0 {
        println!("No branches to delete, you're clean!");
        return;
    }

    let del_opt = if matches.opt_present("l") {
        DeleteOption::Local
    } else if matches.opt_present("r") {
        DeleteOption::Remote
    } else {
        DeleteOption::Both
    };

    let warning_msg = match del_opt {
        DeleteOption::Local => "The following branches will be deleted locally:",
        DeleteOption::Remote => "The following branches will be deleted remotely:",
        DeleteOption::Both => "The following branches will be deleted locally and remotely:"
    };

    println!("{}", warning_msg);
    println!("{}", branches);
    print!("Continue? (yN) ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    match input.to_lowercase().as_ref() {
        "y\n" => (),
        "yes\n" => (),
        _ => return,
    }

    // TODO pass in -l|r options
    match delete_branches(branches, del_opt) {
        Ok(msg) => println!("\n{}", msg),
        Err(msg) => println!("\n{}", msg),
    }
}

fn print_usage(opts: Options) {
    let brief = format!("Usage: git-clean [list] [options]");
    print!("{}", opts.usage(&brief));
}

fn merged_branches() -> String {
    let grep = Command::new("grep")
        .args(&["-vE", "(\\* master|\\smaster)"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| { panic!("ERR: {}", e) });

    let gbranch = Command::new("git")
        .args(&["branch", "--merged", "master"])
        .output()
        .unwrap_or_else(|e| { panic!("ERR: {}", e) });

    {
        grep.stdin.unwrap().write_all(&gbranch.stdout).unwrap();
    }

    let mut s = String::new();
    grep.stdout.unwrap().read_to_string(&mut s).unwrap();
    trim_entries(s)
}

fn delete_branches(branches: String, options: DeleteOption) -> Result<String, String> {
    ensure_master();

    let output = match options {
        DeleteOption::Local => delete_local_branches(&branches).unwrap(),
        DeleteOption::Remote => delete_remote_branches(&branches).unwrap(),
        DeleteOption::Both => {
            let out1 = delete_remote_branches(&branches).unwrap();
            let out2 = delete_local_branches(&branches).unwrap();
            ["Remote:", out1, "Local:", out2].join("\n")
        },
    };

    Ok(output)
}

fn delete_local_branches(branches: &String) -> Result<String, String> {
    let xargs = Command::new("xargs")
        .args(&["git", "branch", "-d"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| { panic!("ERR: {}", e) });

    {
        xargs.stdin.unwrap().write_all(branches.as_bytes()).unwrap()
    }

    let mut s = String::new();
    xargs.stdout.unwrap().read_to_string(&mut s).unwrap();
    Ok(s)
}

fn delete_remote_branches(branches: &String) -> Result<String, String> {
    let xargs = Command::new("xargs")
        .args(&["git", "push", "origin", "--delete"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| { panic!("ERR: {}", e) });

    {
        xargs.stdin.unwrap().write_all(branches.as_bytes()).unwrap()
    }

    let mut s = String::new();
    xargs.stdout.unwrap().read_to_string(&mut s).unwrap();
    Ok(s)
}

fn ensure_master() {
    let current_branch_command = Command::new("git")
        .args(&["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .unwrap_or_else(|e| { panic!("ERR: {}", e) });

    let current_branch = String::from_utf8(current_branch_command.stdout).unwrap();

    if current_branch.trim() != "master" {
        panic!("Please run this command from the master branch");
    }
}

fn trim_entries(entries: String) -> String {
    let split = entries.split("\n");
    let vec: Vec<&str> = split.collect();
    let trimmed_vec: Vec<&str> = vec.iter().map(|s| s.trim()).collect();
    trimmed_vec.join("\n").trim_right_matches("\n").to_owned()
}
