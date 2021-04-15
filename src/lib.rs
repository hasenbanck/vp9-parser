#![warn(missing_docs)]
//! Provides tools to parse VP9 bitstreams and IVF containers.

pub use error::Vp9Error;
pub use ivf::*;

mod error;
mod ivf;

type Result<T> = std::result::Result<T, Vp9Error>;

/// The VP9 profiles.
#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum Vp9Profile {
    /// Color depth: 8 bit/sample, chroma subsampling: 4:2:0
    Profile0,
    /// Color depth: 8 bit, chroma subsampling: 4:2:2, 4:4:0, 4:4:4
    Profile1,
    /// Color depth: 10–12 bit, chroma subsampling: 4:2:0
    Profile2,
    /// Color depth: 10–12 bit, chroma subsampling: 4:2:2, 4:4:0, 4:4:4
    Profile3,
}

/// The VP9 levels.
#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum Vp9Level {
    /// Level 1: 0.20 MBit/s
    Level1,
    /// Level 1.1: 0.80 MBit/s
    Level1_1,
    /// Level 2: 1.8 MBit/s
    Level2,
    /// Level 2.1: 3.6 MBit/s
    Level2_1,
    /// Level 3: 7.2 MBit/s
    Level3,
    /// Level 3.2: 12 MBit/s
    Level3_1,
    /// Level 4: 18 MBit/s
    Level4,
    /// Level 4.1: 30 MBit/s
    Level4_1,
    /// Level 5: 60 MBit/s
    Level5,
    /// Level 5.1: 120 MBit/s
    Level5_1,
    /// Level 5.2: 180 MBit/s
    Level5_2,
    /// Level 6: 180 MBit/s
    Level6,
    /// Level 6.1: 240 MBit/s
    Level6_1,
    /// Level 6.2: 480 MBit/s
    Level6_2,
}

/// A VP9 frame.
pub struct Vp9Frame {
    profile: Vp9Profile,
    level: Vp9Level,
}

impl Vp9Frame {
    /// The profile the frame is using.
    pub fn profile(&self) -> Vp9Profile {
        self.profile
    }

    /// The level the frame is using.
    pub fn level(&self) -> Vp9Level {
        self.level
    }
}

/*
   About reference frames:
   There are REF_FRAMES slots for reference frames (ref_slots[0..7]). Each reference slot contains
   one of the previously decoded frames. Each key frame resets all reference slots with itself.refresh_frame_flags
   from uncompressed header indicates which reference frame slots will be updated with the currently encoded frame,
   e.g. 0b00100010 indicates that slot_1 and slot_5 will reference the current frame.

   (I'm not sure if we need to track this while parsing. Most likely we only need to expose the header fields of the frame.)
*/

/* Super Blocks:
    1. parsing the final byte of the chunk and checking that the superframe_marker equals 0b110 (0xC0)

    superframe_marker f(3)
    bytes_per_framesize_minus_1 f(2)
    frames_in_superframe_minus_1 f(3)

    SzBytes = bytes_per_framesize_minus_1 + 1
    NumFrames = frames_in_superframe_minus_1 + 1

    NOTE – It is legal for a superframe to contain just a single frame and have NumFrames equal to 1.

    2. setting the total size of the superframe_index SzIndex equal to 2 + NumFrames * SzBytes,
    3. checking that the first byte of the superframe_index matches the final byte.

    superframe( sz ) {
        for( i = 0; i < NumFrames; i++ )
            frame( frame_sizes[ i ] )
        superframe_index( )
    }

    superframe_index( ) {
        superframe_header( )
        for( i = 0; i < NumFrames; i++ )
            frame_sizes[i] // Size: SzBytes
        superframe_header( )
    }
*/

/// Parses a VP9 bitstream chunk and returns the encoded frames.
pub fn parse_vp9_chunk(chunk: Vec<u8>) -> Vec<Vp9Frame> {
    vec![]
}
