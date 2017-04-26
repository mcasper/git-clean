use std::fmt::{Display, Error as FmtError, Formatter};
use std::error::Error as StdError;

#[derive(Debug)]
pub enum Error {
    GitInstallationError,
    CurrentBranchInvalidError,
    InvalidRemoteError,
}

use self::Error::*;

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            GitInstallationError => {
                "Unable to execute 'git' on your machine, please make sure it's installed and on \
                 your PATH"
            }
            CurrentBranchInvalidError => {
                "Please make sure to run git-clean from your base branch (defaults to master)."
            }
            InvalidRemoteError => {
                "That remote doesn't exist, please make sure to use a valid remote (defaults to \
                 origin)."
            }
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        self.description().fmt(f)
    }
}
