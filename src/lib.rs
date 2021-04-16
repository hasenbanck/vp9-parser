#![warn(missing_docs)]
//! Provides tools to parse VP9 bitstreams and IVF containers.

use std::convert::TryInto;

pub mod ivf;

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
#[derive(Clone, Debug)]
pub struct Vp9Frame {
    profile: Vp9Profile,
    level: Vp9Level,
    data: Vec<u8>,
}

impl Vp9Frame {
    pub(crate) fn new(data: Vec<u8>) -> Self {
        // TODO
        Self {
            profile: Vp9Profile::Profile0,
            level: Vp9Level::Level1,
            data,
        }
    }

    /// The profile the frame is using.
    pub fn profile(&self) -> Vp9Profile {
        self.profile
    }

    /// The level the frame is using.
    pub fn level(&self) -> Vp9Level {
        self.level
    }

    /// Returns a slice into the data of the uncompressed header.
    pub fn uncompressed_header_data(&self) -> &[u8] {
        // TODO
        &self.data
    }

    /// Returns a slice into the data of the compressed header.
    pub fn compressed_header_data(&self) -> &[u8] {
        // TODO
        &self.data
    }

    /// Returns a slice into the data of the tile with the given index.
    pub fn tile_data(&self, _index: usize) -> Option<&[u8]> {
        // TODO
        None
    }
}

/// Parses a VP9 bitstream chunk and returns the encoded frames.
pub fn parse_vp9_chunk(mut chunk: Vec<u8>) -> Vec<Vp9Frame> {
    if chunk.is_empty() {
        return vec![];
    }

    // Test for a super frame.
    let last_byte_index = chunk.len() - 1;
    let last_byte = chunk[last_byte_index];
    if last_byte & 0b1110_0000 == 0b1100_0000 {
        let bytes_per_framesize_minus_1 = (last_byte & 0b11000) >> 3;
        let frames_in_superframe_minus_1 = last_byte & 0b11;
        let bytes_size = (bytes_per_framesize_minus_1 + 1) as usize;
        let frame_count = (frames_in_superframe_minus_1 + 1) as usize;
        let index_size = 2 + frame_count * bytes_size;
        let first_byte_index = chunk.len() - index_size;
        let first_byte = chunk[first_byte_index];

        // Found a super frame.
        if first_byte == last_byte {
            let mut frames = Vec::with_capacity(frame_count);

            let index_start = first_byte_index + 1;
            let entry_size = frame_count * bytes_size;

            let mut entry_data = Vec::with_capacity(entry_size);
            entry_data.extend_from_slice(&chunk[index_start..index_start + entry_size]);

            for i in 0..frame_count {
                let frame_size = match bytes_size {
                    1 => u8::from_le_bytes(entry_data[i..i + 1].try_into().unwrap()) as usize,
                    2 => u16::from_le_bytes(entry_data[i * 2..(i * 2) + 2].try_into().unwrap())
                        as usize,
                    3 => {
                        let bytes = &entry_data[i * 3..(i * 3) + 3];
                        u32::from_le_bytes([bytes[0], bytes[1], bytes[2], 0x0]) as usize
                    }
                    4 => u32::from_le_bytes(entry_data[i * 4..(i * 4) + 4].try_into().unwrap())
                        as usize,
                    _ => {
                        // Byte size can be at most 4. So this should never trigger.
                        panic!("unsupported byte_size in super frame index")
                    }
                };

                let left_over = chunk.split_off(frame_size);
                let frame = Vp9Frame::new(chunk);
                frames.push(frame);

                chunk = left_over;
            }

            return frames;
        }
    }

    // Normal frame.
    let frame = Vp9Frame::new(chunk);
    vec![frame]
}
