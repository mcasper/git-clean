use commands::{spawn_piped, output, run_command};
use error::GitCleanError;
use getopts::Matches;
use std::io::{Read, Write};

#[derive(Debug)]
pub enum DeleteOption {
    Local,
    Remote,
    Both,
}

pub use self::DeleteOption::*;

impl DeleteOption {
    pub fn new(opts: &Matches) -> DeleteOption {
        if opts.opt_present("l") {
            Local
        } else if opts.opt_present("r") {
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

pub struct GitOptions {
    pub remote: String,
    pub base_branch: String,
    pub squashes: bool,
    pub ignored_branches: Vec<String>,
}

impl GitOptions {
    pub fn new(opts: &Matches) -> GitOptions {
        GitOptions {
            remote: opts.opt_str("R").unwrap_or("origin".to_owned()),
            base_branch: opts.opt_str("b").unwrap_or("master".to_owned()),
            ignored_branches: opts.opt_strs("i"),
            squashes: opts.opt_present("squashes"),
        }
    }

    pub fn validate(&self) -> Result<(), GitCleanError> {
        try!(self.validate_base_branch());
        try!(self.validate_remote());
        Ok(())
    }

    fn validate_base_branch(&self) -> Result<(), GitCleanError> {
        let current_branch = output(&["git", "rev-parse", "--abbrev-ref", "HEAD"]);

        if current_branch != self.base_branch {
            return Err(GitCleanError::CurrentBranchInvalidError);
        };

        Ok(())
    }

    fn validate_remote(&self) -> Result<(), GitCleanError> {
        let grep = spawn_piped(&["grep", &self.remote]);
        let remotes = run_command(&["git", "remote"]);

        {
            grep.stdin.unwrap().write_all(&remotes.stdout).unwrap();
        }

        let mut remote_result = String::new();
        grep.stdout.unwrap().read_to_string(&mut remote_result).unwrap();

        if remote_result.is_empty() {
            return Err(GitCleanError::InvalidRemoteError);
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use getopts::{Options, Matches};
    use super::{DeleteOption, GitOptions};

    // Helpers
    fn parse_args(args: Vec<&str>) -> Matches {
        let mut opts = Options::new();
        opts.optflag("l", "locals", "only delete local branches");
        opts.optflag("r", "remotes", "only delete remote branches");
        opts.optopt("R",
                    "",
                    "changes the git remote used (default is origin)",
                    "REMOTE");
        opts.optopt("b",
                    "",
                    "changes the base for merged branches (default is master)",
                    "BRANCH");
        opts.optmulti("i", "", "ignored branch", "BRANCH");
        opts.optflag("", "squashes", "");
        opts.optflag("h", "help", "print this help menu");

        match opts.parse(&args[..]) {
            Ok(m) => return m,
            Err(_) => panic!("Failed"),
        }
    }

    // DeleteOption tests
    #[test]
    fn test_delete_option_new() {
        let matches = parse_args(vec!["-l"]);

        match DeleteOption::new(&matches) {
            DeleteOption::Local => (),
            other @ _ => panic!("Expected a DeleteOption::Local, but found: {:?}", other),
        };

        let matches = parse_args(vec!["-r"]);

        match DeleteOption::new(&matches) {
            DeleteOption::Remote => (),
            other @ _ => panic!("Expected a DeleteOption::Remote, but found: {:?}", other),
        };

        let matches = parse_args(vec![]);

        match DeleteOption::new(&matches) {
            DeleteOption::Both => (),
            other @ _ => panic!("Expected a DeleteOption::Both, but found: {:?}", other),
        };
    }

    #[test]
    fn test_delete_option_warning_message() {
        assert_eq!("The following branches will be deleted locally:",
                   DeleteOption::Local.warning_message());
        assert_eq!("The following branches will be deleted remotely:",
                   DeleteOption::Remote.warning_message());
        assert_eq!("The following branches will be deleted locally and remotely:",
                   DeleteOption::Both.warning_message());
    }

    // GitOptions tests
    #[test]
    fn test_git_options_new() {
        let matches = parse_args(vec![]);
        let git_options = GitOptions::new(&matches);

        assert_eq!("master".to_owned(), git_options.base_branch);
        assert_eq!("origin".to_owned(), git_options.remote);

        let matches = parse_args(vec!["-b", "stable"]);
        let git_options = GitOptions::new(&matches);

        assert_eq!("stable".to_owned(), git_options.base_branch);
        assert_eq!("origin".to_owned(), git_options.remote);

        let matches = parse_args(vec!["-R", "upstream"]);
        let git_options = GitOptions::new(&matches);

        assert_eq!("master".to_owned(), git_options.base_branch);
        assert_eq!("upstream".to_owned(), git_options.remote);
        assert!(!git_options.squashes);

        let matches = parse_args(vec!["-R", "upstream", "--squashes"]);
        let git_options = GitOptions::new(&matches);

        assert!(git_options.squashes);
    }
}
