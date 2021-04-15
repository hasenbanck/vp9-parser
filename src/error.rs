//! Crate errors.

use std::error::Error;

/// Custom crate errors.
#[derive(Debug)]
pub enum Vp9Error {
    /// A std::io::Error.
    IoError(std::io::Error),
    /// Invalid header.
    InvalidHeader(String),
}

impl std::fmt::Display for Vp9Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Vp9Error::IoError(err) => {
                write!(f, "{:?}", err.source())
            }
            Vp9Error::InvalidHeader(message) => {
                write!(f, "invalid header: {}", message)
            }
        }
    }
}

impl From<std::io::Error> for Vp9Error {
    fn from(err: std::io::Error) -> Vp9Error {
        Vp9Error::IoError(err)
    }
}

impl std::error::Error for Vp9Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
