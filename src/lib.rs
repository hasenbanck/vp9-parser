#![warn(missing_docs)]
//! Provides tools to parse VP9 bitstreams and IVF files.

mod error;
pub mod ivf;

pub use error::Vp9Error;

type Result<T> = std::result::Result<T, Vp9Error>;
