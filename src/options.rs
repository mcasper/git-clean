pub use getopts::{Matches};

#[derive(Debug)]
pub enum DeleteOption {
    Local,
    Remote,
    Both,
}

pub use self::DeleteOption::*;

impl DeleteOption {
    pub fn new(opts: Matches) -> DeleteOption {
        return if opts.opt_present("l") {
            Local
        } else if opts.opt_present("r") {
            Remote
        } else {
            Both
        };
    }

    pub fn warning_message(&self) -> String {
        let source = match self {
            &Local => "locally:",
            &Remote => "remotely:",
            &Both => "locally and remotely:",
        };
        "The following branches will be deleted ".to_owned() + source
    }
}

pub struct GitOptions {
    pub remote: String,
    pub base_branch: String
}

impl GitOptions {
    pub fn new(opts: &Matches) -> GitOptions {
        let remote = match opts.opt_str("R") {
            Some(remote) => remote,
            None => "origin".to_owned(),
        };
        let base_branch = match opts.opt_str("b") {
            Some(branch) => branch,
            None => "master".to_owned(),
        };

        GitOptions {
            remote: remote,
            base_branch: base_branch,
        }
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
        opts.optopt("R", "", "changes the git remote used (default is origin)", "REMOTE");
        opts.optopt("b", "", "changes the base for merged branches (default is master)", "BRANCH");
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

        match DeleteOption::new(matches) {
            DeleteOption::Local => (),
            other @ _ => panic!("Expected a DeleteOption::Local, but found: {:?}", other),
        };

        let matches = parse_args(vec!["-r"]);

        match DeleteOption::new(matches) {
            DeleteOption::Remote => (),
            other @ _ => panic!("Expected a DeleteOption::Remote, but found: {:?}", other),
        };

        let matches = parse_args(vec![]);

        match DeleteOption::new(matches) {
            DeleteOption::Both => (),
            other @ _ => panic!("Expected a DeleteOption::Both, but found: {:?}", other),
        };
    }

    #[test]
    fn test_delete_option_warning_message() {
        assert_eq!("The following branches will be deleted locally:", DeleteOption::Local.warning_message());
        assert_eq!("The following branches will be deleted remotely:", DeleteOption::Remote.warning_message());
        assert_eq!("The following branches will be deleted locally and remotely:", DeleteOption::Both.warning_message());
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
    }
}
