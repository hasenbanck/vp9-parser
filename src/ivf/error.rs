//! IVF errors.

use std::error::Error;

/// Errors that can occur when parsing IVF containers.
#[derive(Debug)]
pub enum IvfError {
    /// A std::io::Error.
    IoError(std::io::Error),
    /// Invalid header.
    InvalidHeader(String),
    /// Unexpected file ending.
    UnexpectedFileEnding,
}

impl std::fmt::Display for IvfError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            IvfError::IoError(err) => {
                write!(f, "{:?}", err.source())
            }
            IvfError::InvalidHeader(message) => {
                write!(f, "invalid header: {}", message)
            }
            IvfError::UnexpectedFileEnding => {
                write!(f, "unexpected file ending")
            }
        }
    }
}

impl From<std::io::Error> for IvfError {
    fn from(err: std::io::Error) -> IvfError {
        IvfError::IoError(err)
    }
}

impl std::error::Error for IvfError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
