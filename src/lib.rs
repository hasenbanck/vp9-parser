#![warn(missing_docs)]
//! Provides tools to parse VP9 bitstreams and IVF containers.

pub use error::Vp9Error;

mod error;
pub mod ivf;

type Result<T> = std::result::Result<T, Vp9Error>;
