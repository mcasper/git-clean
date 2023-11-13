use clap::{App, Arg};

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn build_cli() -> App<'static, 'static> {
    App::new("git-clean")
        .version(VERSION)
        .about("A tool for cleaning old git branches.")
        .arg(
            Arg::with_name("locals")
                .short("l")
                .long("locals")
                .help("Only delete local branches")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("remotes")
                .short("r")
                .long("remotes")
                .help("Only delete remote branches")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("yes")
                .short("y")
                .long("yes")
                .help("Skip the check for deleting branches")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("squashes")
                .short("s")
                .long("squashes")
                .help("Check for squashes by finding branches incompatible with main")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("delete-unpushed-branches")
                .short("d")
                .long("delete-unpushed-branches")
                .help("Delete any local branch that is not present on the remote. Use this to speed up the checks if such branches should always be considered as merged")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("remote")
                .short("R")
                .long("remote")
                .help("Changes the git remote used (default is origin)")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("branch")
                .short("b")
                .long("branch")
                .help("Changes the base for merged branches (default is main)")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("ignore")
                .short("i")
                .long("ignore")
                .help("Ignore given branch (repeat option for multiple branches)")
                .takes_value(true)
                .multiple(true),
        )
}
