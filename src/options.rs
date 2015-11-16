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
    use getopts::{Options};
    use super::DeleteOption;

    // DeleteOption tests
    #[test]
    fn test_delete_option_new() {
        let mut opts = Options::new();
        opts.optflag("l", "local", "only delete local branches");
        opts.optflag("r", "remote", "only delete remote branches");
        opts.optflag("h", "help", "print this help menu");

        // opts throws away the first elem, because it expects it to be the
        // path of the executable
        let args = vec!["./target/debug/git-clean", "-l"];

        let matches = match opts.parse(&args[1..]) {
            Ok(m) => { m }
            Err(_) => { return }
        };

        match DeleteOption::new(matches) {
            DeleteOption::Local => (),
            other @ _ => panic!("Expected a DeleteOption::Local, but found: {:?}", other),
        };

        let args = vec!["./target/debug/git-clean", "-r"];

        let matches = match opts.parse(&args[1..]) {
            Ok(m) => { m }
            Err(_) => { return }
        };

        match DeleteOption::new(matches) {
            DeleteOption::Remote => (),
            other @ _ => panic!("Expected a DeleteOption::Remote, but found: {:?}", other),
        };

        let args = vec!["./target/debug/git-clean"];

        let matches = match opts.parse(&args[1..]) {
            Ok(m) => { m }
            Err(_) => { return }
        };

        match DeleteOption::new(matches) {
            DeleteOption::Both => (),
            other @ _ => panic!("Expected a DeleteOption::Both, but found: {:?}", other),
        };
    }

    #[test]
    fn test_warning_message() {
        assert_eq!("The following branches will be deleted locally:", DeleteOption::Local.warning_message());
        assert_eq!("The following branches will be deleted remotely:", DeleteOption::Remote.warning_message());
        assert_eq!("The following branches will be deleted locally and remotely:", DeleteOption::Both.warning_message());
    }

    // GitOptions tests
}
