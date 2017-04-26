#![deny(warnings)]

extern crate git_clean;

use git_clean::*;

fn main() {
    let matches = cli::build_cli().get_matches();

    run(matches).unwrap_or_else(print_and_exit);
}
