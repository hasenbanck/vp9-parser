//! VP9 parser errors.

use std::error::Error;

/// Errors that can occur when parsing VP9 frames.
#[derive(Debug)]
pub enum ParserError {
    /// A `bitreader::BitReaderError`.
    BitReaderError(bitreader::BitReaderError),
    /// A `std::io::Error`.
    IoError(std::io::Error),
    /// A `TryFromSliceError`.
    TryFromSliceError(std::array::TryFromSliceError),
    /// A `TryFromIntError`.
    TryFromIntError(std::num::TryFromIntError),
    /// Invalid frame marker.
    InvalidFrameMarker,
    /// Invalid padding.
    InvalidPadding,
    /// Invalid sync byte.
    InvalidSyncByte,
    /// Invalid reference frame index.
    InvalidRefFrameIndex,
    /// Invalid metadata.
    InvalidMetadata,
    /// Invalid frame_size byte size.
    InvalidFrameSizeByteSize(usize),
}

impl std::fmt::Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ParserError::BitReaderError(err) => {
                write!(f, "{:?}", err.source())
            }
            ParserError::IoError(err) => {
                write!(f, "{:?}", err.source())
            }
            ParserError::TryFromSliceError(err) => {
                write!(f, "{:?}", err.source())
            }
            ParserError::TryFromIntError(err) => {
                write!(f, "{:?}", err.source())
            }
            ParserError::InvalidFrameMarker => {
                write!(f, "invalid frame marker")
            }
            ParserError::InvalidPadding => {
                write!(f, "invalid padding")
            }
            ParserError::InvalidSyncByte => {
                write!(f, "invalid sync byte")
            }
            ParserError::InvalidRefFrameIndex => {
                write!(f, "invalid reference frame index")
            }
            ParserError::InvalidMetadata => {
                write!(f, "invalid metadata")
            }
            ParserError::InvalidFrameSizeByteSize(size) => {
                write!(f, "invalid frame_size byte size: {}", size)
            }
        }
    }
}

impl From<std::io::Error> for ParserError {
    fn from(err: std::io::Error) -> ParserError {
        ParserError::IoError(err)
    }
}

impl From<std::array::TryFromSliceError> for ParserError {
    fn from(err: std::array::TryFromSliceError) -> ParserError {
        ParserError::TryFromSliceError(err)
    }
}

impl From<std::num::TryFromIntError> for ParserError {
    fn from(err: std::num::TryFromIntError) -> ParserError {
        ParserError::TryFromIntError(err)
    }
}

impl From<bitreader::BitReaderError> for ParserError {
    fn from(err: bitreader::BitReaderError) -> ParserError {
        ParserError::BitReaderError(err)
    }
}

impl std::error::Error for ParserError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            ParserError::IoError(ref e) => Some(e),
            ParserError::TryFromSliceError(ref e) => Some(e),
            ParserError::TryFromIntError(ref e) => Some(e),
            ParserError::BitReaderError(ref e) => Some(e),
            _ => None,
        }
    }
}
