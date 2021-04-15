#![warn(missing_docs)]
//! Provides tools to parse VP9 bitstreams and IVF containers.

pub use error::Vp9Error;

mod error;
pub mod ivf;

type Result<T> = std::result::Result<T, Vp9Error>;

/// Initial conifguration of the Vp9Parser.
pub struct Vp9ParserConfig {
    /// The initial width.
    pub width: u16,
    /// The initial height.
    pub height: u16,
    /// The framerate of the video (frame_rate_rate * frame_rate_scale).
    ///
    /// Example:
    /// 24 fps with a scale of 1000 -> 24000
    pub frame_rate_rate: u32,
    /// Divider of the seconds (integer math).
    pub frame_rate_scale: u32,
}

/// Parses VP9 bitstreams.
pub struct Vp9Parser {
    current_width: u16,
    current_height: u16,
    current_frame_rate_rate: u32,
    current_frame_rate_scale: u32,
}

impl Vp9Parser {
    /// Creates a new Vp9Parser with the given configuration.
    pub fn new(config: &Vp9ParserConfig) -> Self {
        Self {
            current_width: config.width,
            current_height: config.height,
            current_frame_rate_rate: config.frame_rate_rate,
            current_frame_rate_scale: config.frame_rate_scale,
        }
    }
}
