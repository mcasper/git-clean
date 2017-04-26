use commands::{spawn_piped, output, run_command};
use error::Error;
use clap::ArgMatches;
use std::io::{Read, Write};

const DEFAULT_REMOTE: &'static str = "origin";
const DEFAULT_BRANCH: &'static str = "master";

#[derive(Debug)]
pub enum DeleteMode {
    Local,
    Remote,
    Both,
}

pub use self::DeleteMode::*;

impl DeleteMode {
    pub fn new(opts: &ArgMatches) -> DeleteMode {
        if opts.is_present("locals") {
            Local
        } else if opts.is_present("remotes") {
            Remote
        } else {
            Both
        }
    }

    pub fn warning_message(&self) -> String {
        let source = match *self {
            Local => "locally:",
            Remote => "remotely:",
            Both => "locally and remotely:",
        };
        format!("The following branches will be deleted {}", source)
    }
}

pub struct Options {
    pub remote: String,
    pub base_branch: String,
    pub squashes: bool,
    pub ignored_branches: Vec<String>,
    pub delete_mode: DeleteMode,
}

impl Options {
    pub fn new(opts: &ArgMatches) -> Options {
        let default_ignored = Vec::new();
        let ignored = opts.values_of("ignore")
            .map(|i| i.map(|v| v.to_owned()).collect::<Vec<String>>())
            .unwrap_or(default_ignored);
        Options {
            remote: opts.value_of("remote").unwrap_or(DEFAULT_REMOTE).into(),
            base_branch: opts.value_of("branch").unwrap_or(DEFAULT_BRANCH).into(),
            ignored_branches: ignored,
            squashes: opts.is_present("squashes"),
            delete_mode: DeleteMode::new(opts),
        }
    }

    pub fn validate(&self) -> Result<(), Error> {
        self.validate_base_branch()?;
        self.validate_remote()?;
        Ok(())
    }

    fn validate_base_branch(&self) -> Result<(), Error> {
        let current_branch = output(&["git", "rev-parse", "--abbrev-ref", "HEAD"]);

        if current_branch != self.base_branch {
            return Err(Error::CurrentBranchInvalidError);
        };

        Ok(())
    }

    fn validate_remote(&self) -> Result<(), Error> {
        let grep = spawn_piped(&["grep", &self.remote]);
        let remotes = run_command(&["git", "remote"]);

        {
            grep.stdin.unwrap().write_all(&remotes.stdout).unwrap();
        }

        let mut remote_result = String::new();
        grep.stdout.unwrap().read_to_string(&mut remote_result).unwrap();

        if remote_result.is_empty() {
            return Err(Error::InvalidRemoteError);
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use clap;
    use cli;
    use super::{DeleteMode, Options};

    // Helpers
    fn parse_args(args: Vec<&str>) -> clap::ArgMatches {
        cli::build_cli().get_matches_from(args)
    }

    // DeleteMode tests
    #[test]
    fn test_delete_mode_new() {
        let matches = parse_args(vec!["git-clean", "-l"]);

        match DeleteMode::new(&matches) {
            DeleteMode::Local => (),
            other @ _ => panic!("Expected a DeleteMode::Local, but found: {:?}", other),
        };

        let matches = parse_args(vec!["git-clean", "-r"]);

        match DeleteMode::new(&matches) {
            DeleteMode::Remote => (),
            other @ _ => panic!("Expected a DeleteMode::Remote, but found: {:?}", other),
        };

        let matches = parse_args(vec!["git-clean"]);

        match DeleteMode::new(&matches) {
            DeleteMode::Both => (),
            other @ _ => panic!("Expected a DeleteMode::Both, but found: {:?}", other),
        };
    }

    #[test]
    fn test_delete_mode_warning_message() {
        assert_eq!("The following branches will be deleted locally:",
                   DeleteMode::Local.warning_message());
        assert_eq!("The following branches will be deleted remotely:",
                   DeleteMode::Remote.warning_message());
        assert_eq!("The following branches will be deleted locally and remotely:",
                   DeleteMode::Both.warning_message());
    }

    // Options tests
    #[test]
    fn test_git_options_new() {
        let matches = parse_args(vec!["git-clean"]);
        let git_options = Options::new(&matches);

        assert_eq!("master".to_owned(), git_options.base_branch);
        assert_eq!("origin".to_owned(), git_options.remote);

        let matches = parse_args(vec!["git-clean", "-b", "stable"]);
        let git_options = Options::new(&matches);

        assert_eq!("stable".to_owned(), git_options.base_branch);
        assert_eq!("origin".to_owned(), git_options.remote);

        let matches = parse_args(vec!["git-clean", "-R", "upstream"]);
        let git_options = Options::new(&matches);

        assert_eq!("master".to_owned(), git_options.base_branch);
        assert_eq!("upstream".to_owned(), git_options.remote);
        assert!(!git_options.squashes);

        let matches = parse_args(vec!["git-clean", "-R", "upstream", "--squashes"]);
        let git_options = Options::new(&matches);

        assert!(git_options.squashes);
    }
}
