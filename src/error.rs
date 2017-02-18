use std::{error, fmt};
use std::error::Error;

#[derive(Debug)]
pub enum GitCleanError {
    GitInstallationError,
    CurrentBranchInvalidError,
    InvalidRemoteError,
}

use self::GitCleanError::*;

impl error::Error for GitCleanError {
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

impl fmt::Display for GitCleanError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.description().fmt(f)
    }
}
