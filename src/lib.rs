#![deny(warnings)]

extern crate clap;

pub mod cli;

use clap::ArgMatches;

mod branches;
use branches::Branches;

mod commands;
pub use commands::validate_git_installation;

mod error;
use error::Error;

mod options;
use options::Options;

pub fn run(matches: &ArgMatches) -> Result<(), error::Error> {
    validate_git_installation()?;

    let options = Options::new(matches);
    options.validate()?;

    let branches = Branches::merged(&options);

    if branches.string.is_empty() {
        println!("No branches to delete, you're clean!");
        return Ok(());
    }

    if !matches.is_present("yes") {
        branches.print_warning_and_prompt(&options.delete_mode)?;
    }

    let msg = branches.delete(&options);
    println!("\n{}", msg);

    Ok(())
}

pub fn print_and_exit(error: &Error) {
    println!("{}", error);
    std::process::exit(1);
}
