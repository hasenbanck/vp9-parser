//! VP9 parser errors.

use std::error::Error;

/// Errors that can occur when parsing VP9 frames.
#[derive(Debug)]
pub enum Vp9ParserError {
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

impl std::fmt::Display for Vp9ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Vp9ParserError::BitReaderError(err) => {
                write!(f, "{:?}", err.source())
            }
            Vp9ParserError::IoError(err) => {
                write!(f, "{:?}", err.source())
            }
            Vp9ParserError::TryFromSliceError(err) => {
                write!(f, "{:?}", err.source())
            }
            Vp9ParserError::TryFromIntError(err) => {
                write!(f, "{:?}", err.source())
            }
            Vp9ParserError::InvalidFrameMarker => {
                write!(f, "invalid frame marker")
            }
            Vp9ParserError::InvalidPadding => {
                write!(f, "invalid padding")
            }
            Vp9ParserError::InvalidSyncByte => {
                write!(f, "invalid sync byte")
            }
            Vp9ParserError::InvalidRefFrameIndex => {
                write!(f, "invalid reference frame index")
            }
            Vp9ParserError::InvalidMetadata => {
                write!(f, "invalid metadata")
            }
            Vp9ParserError::InvalidFrameSizeByteSize(size) => {
                write!(f, "invalid frame_size byte size: {}", size)
            }
        }
    }
}

impl From<std::io::Error> for Vp9ParserError {
    fn from(err: std::io::Error) -> Vp9ParserError {
        Vp9ParserError::IoError(err)
    }
}

impl From<std::array::TryFromSliceError> for Vp9ParserError {
    fn from(err: std::array::TryFromSliceError) -> Vp9ParserError {
        Vp9ParserError::TryFromSliceError(err)
    }
}

impl From<std::num::TryFromIntError> for Vp9ParserError {
    fn from(err: std::num::TryFromIntError) -> Vp9ParserError {
        Vp9ParserError::TryFromIntError(err)
    }
}

impl From<bitreader::BitReaderError> for Vp9ParserError {
    fn from(err: bitreader::BitReaderError) -> Vp9ParserError {
        Vp9ParserError::BitReaderError(err)
    }
}

impl std::error::Error for Vp9ParserError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Vp9ParserError::IoError(ref e) => Some(e),
            Vp9ParserError::TryFromSliceError(ref e) => Some(e),
            Vp9ParserError::BitReaderError(ref e) => Some(e),
            _ => None,
        }
    }
}
