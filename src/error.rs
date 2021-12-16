use std::error::Error as StdError;
use std::fmt::{Display, Error as FmtError, Formatter};
use std::io::Error as IoError;

#[derive(Debug)]
pub enum Error {
    GitInstallation,
    CurrentBranchInvalid,
    InvalidRemote,
    ExitEarly,
    Io(IoError),
}

use self::Error::*;

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match *self {
            Io(ref io_error) => Some(io_error),
            _ => None,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        match *self {
            Io(ref io_error) => io_error.fmt(f),
            ExitEarly => Ok(()),
            GitInstallation => {
                write!(f, "Unable to execute 'git' on your machine, please make sure it's installed and on your PATH")
            }
            CurrentBranchInvalid => {
                write!(
                    f,
                    "Please make sure to run git-clean from your base branch (defaults to main)."
                )
            }
            InvalidRemote => {
                write!(f, "That remote doesn't exist, please make sure to use a valid remote (defaults to origin).")
            }
        }
    }
}

impl From<IoError> for Error {
    fn from(error: IoError) -> Error {
        Io(error)
    }
}
