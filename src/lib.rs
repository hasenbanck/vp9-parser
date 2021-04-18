#![warn(missing_docs)]
//! Provides tools to parse VP9 bitstreams and IVF containers.

use std::convert::TryInto;

use bitreader::BitReader;

pub use error::Vp9ParserError;

mod error;
pub mod ivf;

type Result<T> = std::result::Result<T, Vp9ParserError>;

// Number of segments allowed in segmentation map.
const MAX_SEGMENTS: usize = 8;

const INTRA_FRAME: usize = 0;
const LAST_FRAME: usize = 1;
const GOLDEN_FRAME: usize = 2;
const ALTREF_FRAME: usize = 3;

/// The segmentation features.
#[derive(Clone, Copy, Debug, Default)]
pub struct SegmentFeatures {
    /// Quantizer segment feature.
    pub alt_q: bool,
    /// Loop filter segment feature.
    pub alt_l: bool,
    /// Reference frame segment feature.
    pub ref_frame: bool,
    /// Skip segment feature.
    pub skip_segment: bool,
}

impl From<SegmentFeatures> for [bool; 4] {
    fn from(f: SegmentFeatures) -> Self {
        [f.alt_q, f.alt_l, f.ref_frame, f.skip_segment]
    }
}

/// The segmentation feature values.
#[derive(Clone, Copy, Debug, Default)]
pub struct SegmentFeatureValues {
    /// Quantizer segment feature value.
    pub alt_q: i16,
    /// Loop filter segment feature value.
    pub alt_l: i16,
    /// Reference frame segment feature value.
    pub ref_frame: i16,
    // Skip frame segment feature value is always 0.
}

impl From<SegmentFeatureValues> for [i16; 4] {
    fn from(f: SegmentFeatureValues) -> Self {
        [f.alt_q, f.alt_l, f.ref_frame, 0]
    }
}

/// The VP9 profiles.
#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum Profile {
    /// Color depth: 8 bit/sample, chroma subsampling: 4:2:0
    Profile0,
    /// Color depth: 8 bit, chroma subsampling: 4:2:2, 4:4:0, 4:4:4
    Profile1,
    /// Color depth: 10–12 bit, chroma subsampling: 4:2:0
    Profile2,
    /// Color depth: 10–12 bit, chroma subsampling: 4:2:2, 4:4:0, 4:4:4
    Profile3,
}

impl From<u8> for Profile {
    fn from(i: u8) -> Self {
        match i {
            0 => Profile::Profile0,
            1 => Profile::Profile1,
            2 => Profile::Profile2,
            3 => Profile::Profile3,
            _ => {
                panic!("unhandled profile")
            }
        }
    }
}

impl From<Profile> for u8 {
    fn from(p: Profile) -> Self {
        match p {
            Profile::Profile0 => 0,
            Profile::Profile1 => 1,
            Profile::Profile2 => 2,
            Profile::Profile3 => 3,
        }
    }
}

/// Chroma subsampling.
pub enum Subsampling {
    /// 4:4:4 - No chrome subsampling
    Yuv444,
    /// 4:4:0 - Subsampling along the y axis.
    Yuv440,
    /// 4:2:2 - Subsampling along the x axis.
    Yuv422,
    /// 4:2:0 - Subsampling along both x and y axis.
    Yuv420,
}

/// Color space.
#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum ColorSpace {
    /// Unknown (in this case the color space must be signaled outside the VP9 bitstream).
    Unknown,
    /// Rec. ITU-R BT.601-7
    Bt601,
    /// Rec. ITU-R BT.709-6
    Bt709,
    /// SMPTE-170
    Smpte170,
    /// SMPTE-240
    Smpte240,
    /// Rec. ITU-R BT.2020-2
    Bt2020,
    /// Reserved
    Reserved,
    /// sRGB (IEC 61966-2-1)
    Rgb,
}

impl From<u8> for ColorSpace {
    fn from(i: u8) -> Self {
        match i {
            0 => ColorSpace::Unknown,
            1 => ColorSpace::Bt601,
            2 => ColorSpace::Bt709,
            3 => ColorSpace::Smpte170,
            4 => ColorSpace::Smpte240,
            5 => ColorSpace::Bt2020,
            6 => ColorSpace::Reserved,
            7 => ColorSpace::Rgb,
            _ => panic!("unhandled color space"),
        }
    }
}

/// Color depth.
#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum ColorDepth {
    /// 8 bit depth.
    Depth8,
    /// 10 bit depth.
    Depth10,
    /// 12 bit depth.
    Depth12,
}

/// Specifies the black level and range of the luma and chroma signals as specified in
/// Rec. ITU-R BT.709-6 and Rec. ITU-R BT.2020-2.
#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum ColorRange {
    /// Put restriction on Y, U, V values.
    StudioSwing,
    /// No restriction on Y, U, V values.
    FullSwing,
}

impl From<bool> for ColorRange {
    fn from(b: bool) -> Self {
        match b {
            false => ColorRange::StudioSwing,
            true => ColorRange::FullSwing,
        }
    }
}

/// Type of the interpolation filter.
#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum InterpolationFilter {
    /// EIGHTTAP.
    Eighttap,
    /// EIGHTTAP_SMOOTH.
    EighttapSmooth,
    /// EIGHTTAP_SHARP.
    EighttapSharp,
    /// BILINEAR.
    Bilinear,
    /// SWITCHABLE.
    Switchable,
}

/// The type of a frame.
#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum FrameType {
    /// Frame is a key frame.
    KeyFrame,
    /// Frame is not a key frame.
    NonKeyFrame,
}

/// Defines if the frame context should be reset.
#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum ResetFrameContext {
    /// Do not reset any frame context.
    No0,
    /// Do not reset any frame context.
    No1,
    /// Resets just the context specified in the frame header.
    SingleReset,
    /// Resets all contexts.
    FullReset,
}

impl From<u8> for ResetFrameContext {
    fn from(i: u8) -> Self {
        match i {
            0 => ResetFrameContext::No0,
            1 => ResetFrameContext::No1,
            2 => ResetFrameContext::SingleReset,
            3 => ResetFrameContext::FullReset,
            _ => panic!("unhandled reset context"),
        }
    }
}

/// A VP9 frame.
#[derive(Clone, Debug)]
pub struct Frame {
    data: Vec<u8>,
    profile: Profile,
    show_existing_frame: bool,
    frame_to_show_map_idx: Option<u8>,
    last_frame_type: FrameType,
    frame_type: FrameType,
    show_frame: bool,
    error_resilient_mode: bool,
    intra_only: bool,
    reset_frame_context: ResetFrameContext,
    ref_frame_indices: [u8; 3],
    ref_frame_sign_bias: [bool; 4],
    allow_high_precision_mv: bool,
    refresh_frame_context: bool,
    refresh_frame_flags: u8,
    frame_parallel_decoding_mode: bool,
    frame_context_idx: u8,
    uncompressed_header_size: usize,
    compressed_header_size: usize,
    tile_size: usize,
    color_depth: ColorDepth,
    color_space: ColorSpace,
    color_range: ColorRange,
    subsampling_x: bool,
    subsampling_y: bool,
    width: u16,
    height: u16,
    render_width: u16,
    render_height: u16,
    mi_cols: u16,
    mi_rows: u16,
    tile_rows_log2: u8,
    tile_cols_log2: u8,
    interpolation_filter: InterpolationFilter,
    loop_filter_level: u8,
    loop_filter_sharpness: u8,
    loop_filter_delta_enabled: bool,
    update_ref_delta: bool,
    loop_filter_ref_deltas: [i8; 4],
    update_mode_delta: bool,
    loop_filter_mode_deltas: [i8; 2],
    base_q_idx: i32,
    delta_q_y_dc: i32,
    delta_q_uv_dc: i32,
    delta_q_uv_ac: i32,
    lossless: bool,
    segmentation_enabled: bool,
    segmentation_update_map: bool,
    segment_tree_probs: [u8; 7],
    segment_pred_probs: [u8; 3],
    segmentation_temporal_update: bool,
    segmentation_update_data: bool,
    segmentation_abs_or_delta_update: bool,

    segment_feature_active: [SegmentFeatures; 8],
    segment_feature_data: [SegmentFeatureValues; 8],
}

impl Default for Frame {
    fn default() -> Self {
        Self {
            data: vec![],
            show_existing_frame: false,
            frame_to_show_map_idx: None,
            profile: Profile::Profile0,
            last_frame_type: FrameType::KeyFrame,
            frame_type: FrameType::KeyFrame,
            show_frame: false,
            error_resilient_mode: false,
            intra_only: false,
            reset_frame_context: ResetFrameContext::No0,
            refresh_frame_flags: 0,
            ref_frame_indices: [0u8; 3],
            ref_frame_sign_bias: [false; 4],
            allow_high_precision_mv: false,
            refresh_frame_context: false,
            frame_parallel_decoding_mode: true,
            frame_context_idx: 0,
            uncompressed_header_size: 0,
            compressed_header_size: 0,
            tile_size: 0,
            color_depth: ColorDepth::Depth8,
            color_space: ColorSpace::Unknown,
            color_range: ColorRange::StudioSwing,
            subsampling_x: true,
            subsampling_y: true,
            width: 0,
            height: 0,
            render_width: 0,
            render_height: 0,
            mi_cols: 0,
            mi_rows: 0,
            tile_rows_log2: 0,
            tile_cols_log2: 0,
            interpolation_filter: InterpolationFilter::Eighttap,
            loop_filter_level: 0,
            loop_filter_sharpness: 0,
            loop_filter_delta_enabled: false,
            update_ref_delta: false,
            loop_filter_ref_deltas: [0i8; 4],
            update_mode_delta: false,
            loop_filter_mode_deltas: [0i8; 2],
            base_q_idx: 0,
            delta_q_y_dc: 0,
            delta_q_uv_dc: 0,
            delta_q_uv_ac: 0,
            lossless: false,
            segmentation_enabled: false,
            segmentation_update_map: false,
            segment_tree_probs: [0u8; 7],
            segment_pred_probs: [0u8; 3],
            segmentation_temporal_update: false,
            segmentation_update_data: false,
            segmentation_abs_or_delta_update: false,
            segment_feature_active: [Default::default(); 8],
            segment_feature_data: [Default::default(); 8],
        }
    }
}

impl Frame {
    /// Returns a slice into the data of the compressed header.
    pub fn compressed_header_data(&self) -> &[u8] {
        &self.data[self.uncompressed_header_size
            ..self.uncompressed_header_size + self.compressed_header_size]
    }

    /// Returns a slice into the data of the compressed header and tile data.
    pub fn compressed_header_and_tile_data(&self) -> &[u8] {
        &self.data[self.uncompressed_header_size..self.data.len()]
    }

    /// Returns a slice into the data of the tile data.
    pub fn tile_data(&self) -> &[u8] {
        &self.data[self.uncompressed_header_size + self.compressed_header_size..self.data.len()]
    }

    /// The profile the frame is using.
    pub fn profile(&self) -> Profile {
        self.profile
    }

    /// Indicates that the frame indexed by `frame_to_show_map_idx` is to be displayed.
    /// The frame contains no actual frame data.
    pub fn show_existing_frame(&self) -> bool {
        self.show_existing_frame
    }

    /// Specifies the frame to be displayed. It is only available if `show_existing_frame` is true.
    pub fn frame_to_show_map_idx(&self) -> Option<u8> {
        self.frame_to_show_map_idx
    }

    /// The frame type of the previous frame.
    pub fn last_frame_type(&self) -> FrameType {
        self.last_frame_type
    }

    /// The frame type of this frame.
    pub fn frame_type(&self) -> FrameType {
        self.frame_type
    }

    /// Indicates that error resilient mode is enabled.
    ///
    /// Error resilient mode allows the syntax of a frame to be decoded
    /// independently of previous frames.
    pub fn error_resilient_mode(&self) -> bool {
        self.error_resilient_mode
    }

    /// Indicates that a frame is an `intra-only` frame.
    ///
    /// A key frame is different to an `intra-only` frame even though both only use
    /// intra prediction. The difference is that a key frame fully resets the decoding process.
    pub fn intra_only(&self) -> bool {
        self.intra_only
    }

    /// Specifies whether the frame context should be reset to default values.
    pub fn reset_frame_context(&self) -> FrameType {
        self.frame_type
    }

    /// The indices of the used reference frames.
    pub fn ref_frame_indices(&self) -> &[u8; 3] {
        &self.ref_frame_indices
    }

    /// Last reference frame index.
    pub fn last_ref_frame_index(&self) -> u8 {
        self.ref_frame_indices[0]
    }

    /// Golden reference frame index.
    pub fn golden_ref_frame_index(&self) -> u8 {
        self.ref_frame_indices[1]
    }

    /// Alternate reference frame index.
    pub fn alt_ref_frame_index(&self) -> u8 {
        self.ref_frame_indices[2]
    }

    /// Specifies the intended direction of the motion vector in time for each reference frame.
    pub fn ref_frame_sign_bias(&self) -> &[bool; 4] {
        &self.ref_frame_sign_bias
    }

    /// Specifies the precision of the motion vectors.
    ///
    /// False = quarter precision, True = eighth precision.
    pub fn allow_high_precision_mv(&self) -> bool {
        self.allow_high_precision_mv
    }

    /// Specifies that the probabilities computed for this frame
    /// should be stored for reference by future frames.
    pub fn refresh_frame_context(&self) -> bool {
        self.refresh_frame_context
    }

    /// Contains a bitmask that specifies which reference frame slots
    /// will be updated with the current frame after it is decoded.
    ///
    /// First bit = first frame (1). Last bit = last frame (8).
    pub fn refresh_frame_flags(&self) -> u8 {
        self.refresh_frame_flags
    }

    /// Specifies if parallel decoding mode is activated.
    pub fn frame_parallel_decoding_mode(&self) -> bool {
        self.frame_parallel_decoding_mode
    }

    /// Specifies which frame context to use.
    pub fn frame_context_idx(&self) -> u8 {
        self.frame_context_idx
    }

    /// The size of the uncompressed header.
    pub fn uncompressed_header_size(&self) -> usize {
        self.uncompressed_header_size
    }

    /// The size of the uncompressed header.
    pub fn compressed_header_size(&self) -> usize {
        self.compressed_header_size
    }

    /// The size of the tile data.
    pub fn tile_size(&self) -> usize {
        self.tile_size
    }

    /// The color depth of the frame.
    pub fn color_depth(&self) -> ColorDepth {
        self.color_depth
    }

    /// The color space of the frame.
    pub fn color_space(&self) -> ColorSpace {
        self.color_space
    }

    /// The color range of the frame.
    pub fn color_range(&self) -> ColorRange {
        self.color_range
    }

    /// The subsampling the frame is using.
    pub fn subsampling(&self) -> Subsampling {
        if !self.subsampling_x && !self.subsampling_y {
            Subsampling::Yuv444
        } else if !self.subsampling_x && self.subsampling_y {
            Subsampling::Yuv440
        } else if self.subsampling_x && !self.subsampling_y {
            Subsampling::Yuv422
        } else {
            Subsampling::Yuv420
        }
    }

    /// Indicates if sub sampling is used along the x axis.
    pub fn subsampling_x(&self) -> bool {
        self.subsampling_x
    }

    /// Indicates if sub sampling is used along the y axis.
    pub fn subsampling_y(&self) -> bool {
        self.subsampling_y
    }

    /// The width of the frame.
    pub fn width(&self) -> u16 {
        self.width
    }

    /// The height of the frame.
    pub fn height(&self) -> u16 {
        self.height
    }

    /// A hint for the application for the desired width to render.
    pub fn render_width(&self) -> u16 {
        self.render_width
    }

    /// A hint for the application for the desired height to render.
    pub fn render_height(&self) -> u16 {
        self.render_height
    }

    /// A variable holding the vertical location of the block in units of 8x8 pixels.
    pub fn mi_cols(&self) -> u16 {
        self.mi_cols
    }

    /// A variable holding the horizontal location of the block in units of 8x8 pixels.
    pub fn mi_rows(&self) -> u16 {
        self.mi_rows
    }

    /// The base 2 logarithm of the height of each tile (where the height is measured in units
    /// of 8x8 blocks)
    pub fn tile_rows_log2(&self) -> u8 {
        self.tile_rows_log2
    }

    /// The base 2 logarithm of the width of each tile (where the width is measured in units
    /// of 8x8 blocks)
    pub fn tile_cols_log2(&self) -> u8 {
        self.tile_cols_log2
    }

    /// The type of filter used in inter prediction.
    pub fn interpolation_filter(&self) -> InterpolationFilter {
        self.interpolation_filter
    }

    /// The loop filter strength.
    pub fn loop_filter_level(&self) -> u8 {
        self.loop_filter_level
    }

    /// The loop filter sharpness.
    pub fn loop_filter_sharpness(&self) -> u8 {
        self.loop_filter_sharpness
    }

    /// Indicates that the filter level depends on the mode and reference frame
    /// used to predict a block.
    pub fn loop_filter_delta_enabled(&self) -> bool {
        self.loop_filter_delta_enabled
    }

    /// Indicates that the the bitstream contains the syntax element loop_filter_ref_delta.
    pub fn update_ref_delta(&self) -> bool {
        self.update_ref_delta
    }

    /// Contains the adjustment needed for the filter level based on the chosen reference frame.
    pub fn loop_filter_ref_deltas(&self) -> &[i8; 4] {
        &self.loop_filter_ref_deltas
    }

    /// Indicates that the the bitstream contains the syntax element loop_filter_mode_deltas.
    pub fn update_mode_delta(&self) -> bool {
        self.update_mode_delta
    }

    /// Contains the adjustment needed for the filter level based on the chosen mode.
    pub fn loop_filter_mode_deltas(&self) -> &[i8; 2] {
        &self.loop_filter_mode_deltas
    }

    /// The base frame qindex. This is used for Y AC coefficients and as the base value
    /// for the other quantizers.
    pub fn base_q_idx(&self) -> i32 {
        self.base_q_idx
    }

    /// The Y DC quantizer relative to base_q_idx.
    pub fn delta_q_y_dc(&self) -> i32 {
        self.delta_q_y_dc
    }

    /// The UV DC quantizer relative to base_q_idx.
    pub fn delta_q_uv_dc(&self) -> i32 {
        self.delta_q_uv_dc
    }

    /// The UV AC quantizer relative to base_q_idx.
    pub fn delta_q_uv_ac(&self) -> i32 {
        self.delta_q_uv_ac
    }

    /// Indicates that the frame is coded using a special 4x4 transform designed
    /// for encoding frames that are bit-identical with the original frames.
    pub fn lossless(&self) -> bool {
        self.lossless
    }

    /// Specifies that this frame makes use of the segmentation tool.
    pub fn segmentation_enabled(&self) -> bool {
        self.segmentation_enabled
    }

    /// Specifies that the segmentation map should be updated during the decoding of this frame.
    pub fn segmentation_update_map(&self) -> bool {
        self.segmentation_update_map
    }

    /// The probability values to be used when decoding segment_id.
    pub fn segment_tree_probs(&self) -> &[u8; 7] {
        &self.segment_tree_probs
    }

    /// The probability values to be used when decoding seg_id_predicted.
    pub fn segment_pred_probs(&self) -> &[u8; 3] {
        &self.segment_pred_probs
    }

    /// Indicates that the updates to the segmentation map are coded
    /// relative to the existing segmentation map.
    pub fn segmentation_temporal_update(&self) -> bool {
        self.segmentation_temporal_update
    }

    /// Indicates that new parameters are about to be specified for each segment.
    pub fn segmentation_update_data(&self) -> bool {
        self.segmentation_update_data
    }

    /// Indicates that the segmentation parameters represent the actual values to be used,
    /// otherwise the segmentation parameters represent adjustments relative to the standard values.
    pub fn segmentation_abs_or_delta_update(&self) -> bool {
        self.segmentation_abs_or_delta_update
    }

    /// Indicates that the corresponding feature is used for the reference frame.
    pub fn segment_feature_active(&self) -> &[SegmentFeatures; 8] {
        &self.segment_feature_active
    }

    /// Specifies the value of the feature data for a segment feature.
    pub fn segment_feature_data(&self) -> &[SegmentFeatureValues; 8] {
        &self.segment_feature_data
    }
}

/// Parses VP9 bitstreams.
#[derive(Clone, Debug)]
pub struct Vp9Parser {
    // States that need to be tracked between frames.
    last_frame_type: FrameType,
    ref_frame_sizes: [(u16, u16); 8],
    loop_filter_ref_deltas: [i8; 4],
    loop_filter_mode_deltas: [i8; 2],
}

impl Default for Vp9Parser {
    fn default() -> Self {
        Self {
            last_frame_type: FrameType::NonKeyFrame,
            ref_frame_sizes: [(0u16, 0u16); 8],
            loop_filter_ref_deltas: [1, 0, -1, -1],
            loop_filter_mode_deltas: [0, 0],
        }
    }
}

impl Vp9Parser {
    /// Creates a new parser.
    pub fn new() -> Self {
        Default::default()
    }

    /// Resets the state of the parser. Used when switching the bitstream or seeking.
    pub fn reset(&mut self) {
        self.last_frame_type = FrameType::NonKeyFrame
    }

    /// Parses a VP9 bitstream packet and returns the encoded frames.
    ///
    /// Packets needs to be supplied in the order they are appearing in the bitstream. The caller
    /// needs to reset the parser if the bitstream is changed or a seek happened. Not resetting the
    /// parser in such cases results in undefined behavior of the decoder.
    pub fn parse_vp9_packet(&mut self, mut packet: Vec<u8>) -> Result<Vec<Frame>> {
        if packet.is_empty() {
            return Ok(vec![]);
        }

        // Test for a super frame.
        let last_byte_index = packet.len() - 1;
        let last_byte = packet[last_byte_index];
        if last_byte & 0b1110_0000 == 0b1100_0000 {
            let bytes_per_framesize_minus_1 = (last_byte & 0b11000) >> 3;
            let frames_in_superframe_minus_1 = last_byte & 0b111;
            let bytes_size = (bytes_per_framesize_minus_1 + 1) as usize;
            let frame_count = (frames_in_superframe_minus_1 + 1) as usize;
            let index_size = 2 + frame_count * bytes_size;
            let first_byte_index = packet.len() - index_size;
            let first_byte = packet[first_byte_index];

            // Found a super frame.
            if first_byte == last_byte {
                let mut frames = Vec::with_capacity(frame_count);

                let index_start = first_byte_index + 1;
                let entry_size = frame_count * bytes_size;

                let mut entry_data = Vec::with_capacity(entry_size);
                entry_data.extend_from_slice(&packet[index_start..index_start + entry_size]);

                match frame_count {
                    1 => {
                        // Odd, but valid bitstream configuration.
                        let frame_size = self.read_frame_size(&mut entry_data, bytes_size, 0);
                        packet.truncate(frame_size);
                        let frame = self.parse_vp9_frame(packet)?;

                        frames.push(frame);
                    }
                    2 => {
                        // Most common case. The first frame produces a frame that is not displayed but
                        // stored as a reference frame. The second frame is mostly empty and references
                        // the previously stored frame.
                        let frame_size = self.read_frame_size(&mut entry_data, bytes_size, 0);
                        let mut left_over = packet.split_off(frame_size);
                        let first_frame = self.parse_vp9_frame(packet)?;

                        let frame_size = self.read_frame_size(&mut entry_data, bytes_size, 1);
                        left_over.truncate(frame_size);
                        let second_frame = self.parse_vp9_frame(left_over)?;

                        frames.push(first_frame);
                        frames.push(second_frame);
                    }
                    _ => {
                        // Odd, but also a valid bitstream configuration.
                        for frame_index in 0..frame_count {
                            let frame_size =
                                self.read_frame_size(&mut entry_data, bytes_size, frame_index);

                            let left_over = packet.split_off(frame_size);
                            let frame = self.parse_vp9_frame(packet)?;
                            frames.push(frame);

                            packet = left_over;
                        }
                    }
                }

                return Ok(frames);
            }
        }

        // Normal frame.
        let frame = self.parse_vp9_frame(packet)?;
        Ok(vec![frame])
    }

    fn read_frame_size(&self, entry_data: &mut Vec<u8>, bytes_size: usize, index: usize) -> usize {
        // sic! Even though the values inside the uncompressed header are saved in BE,
        // these values are saved in LE.
        match bytes_size {
            1 => u8::from_le_bytes(entry_data[index..index + 1].try_into().unwrap()) as usize,
            2 => u16::from_le_bytes(entry_data[index * 2..(index * 2) + 2].try_into().unwrap())
                as usize,
            3 => {
                let bytes = &entry_data[index * 3..(index * 3) + 3];
                u32::from_le_bytes([bytes[0], bytes[1], bytes[2], 0x0]) as usize
            }
            4 => u32::from_le_bytes(entry_data[index * 4..(index * 4) + 4].try_into().unwrap())
                as usize,
            _ => {
                // Byte size can be at most 4. So this should never trigger.
                panic!("unsupported byte_size in super frame index")
            }
        }
    }

    fn parse_vp9_frame(&mut self, data: Vec<u8>) -> Result<Frame> {
        let mut br = BitReader::new(&data);

        let mut frame = Frame {
            ..Default::default()
        };

        let frame_marker = br.read_u8(2)?;
        if frame_marker != 2 {
            return Err(Vp9ParserError::InvalidFrameMarker);
        }

        let profile_low_bit = br.read_u8(1)?;
        let profile_high_bit = br.read_u8(1)?;
        frame.profile = ((profile_high_bit << 1) + profile_low_bit).into();
        if frame.profile == Profile::Profile3 {
            let _reserved_zero = br.read_u8(1)?;
        }

        frame.show_existing_frame = br.read_bool()?;
        if frame.show_existing_frame {
            frame.frame_to_show_map_idx = Some(br.read_u8(3)?);
            return Ok(frame);
        }

        frame.last_frame_type = self.last_frame_type;
        if br.read_bool()? {
            frame.frame_type = FrameType::NonKeyFrame
        };
        self.last_frame_type = frame.frame_type;

        frame.show_frame = br.read_bool()?;
        frame.error_resilient_mode = br.read_bool()?;

        if frame.frame_type == FrameType::KeyFrame {
            self.frame_sync_code(&mut br)?;
            self.color_config(&mut br, &mut frame)?;
            self.frame_size(&mut br, &mut frame)?;
            self.render_size(&mut br, &mut frame)?;
            frame.refresh_frame_flags = 0xFF;
        } else {
            if !frame.show_frame {
                frame.intra_only = br.read_bool()?
            };

            if !frame.error_resilient_mode {
                frame.reset_frame_context = br.read_u8(2)?.into()
            };

            if frame.intra_only {
                self.frame_sync_code(&mut br)?;
                if frame.profile > Profile::Profile0 {
                    self.color_config(&mut br, &mut frame)?;
                } else {
                    frame.color_space = ColorSpace::Bt601;
                }
                frame.refresh_frame_flags = br.read_u8(8)?;
                self.frame_size(&mut br, &mut frame)?;
                self.render_size(&mut br, &mut frame)?;
            } else {
                frame.refresh_frame_flags = br.read_u8(8)?;
                for i in 0..3 {
                    frame.ref_frame_indices[i] = br.read_u8(3)?;
                    frame.ref_frame_sign_bias[LAST_FRAME + i] = br.read_bool()?;
                }
                self.frame_size_with_refs(&mut br, &mut frame)?;
                frame.allow_high_precision_mv = br.read_bool()?;
                self.read_interpolation_filter(&mut br, &mut frame)?;
            }
        }

        if !frame.error_resilient_mode {
            frame.refresh_frame_context = br.read_bool()?;
            frame.frame_parallel_decoding_mode = br.read_bool()?;
        };
        frame.frame_context_idx = br.read_u8(2)?;

        if frame.intra_only || frame.error_resilient_mode {
            frame.frame_context_idx = 0
        }

        if frame.frame_type == FrameType::KeyFrame || frame.error_resilient_mode || frame.intra_only
        {
            // Reset the loop filter deltas.
            self.loop_filter_ref_deltas[INTRA_FRAME] = 1;
            self.loop_filter_ref_deltas[LAST_FRAME] = 0;
            self.loop_filter_ref_deltas[GOLDEN_FRAME] = -1;
            self.loop_filter_ref_deltas[ALTREF_FRAME] = -1;
            self.loop_filter_mode_deltas[0] = 0;
            self.loop_filter_mode_deltas[1] = 0;
        }
        self.loop_filter_params(&mut br, &mut frame)?;

        frame.loop_filter_ref_deltas = self.loop_filter_ref_deltas;
        frame.loop_filter_mode_deltas = self.loop_filter_mode_deltas;

        self.quantization_params(&mut br, &mut frame)?;
        self.segmentation_params(&mut br, &mut frame)?;
        self.tile_info(&mut br, &mut frame)?;

        frame.compressed_header_size = br.read_u16(16)? as usize;
        self.trailing_bits(&mut br)?;
        frame.uncompressed_header_size = (br.position() / 8) as usize;

        drop(br);
        frame.data = data;

        frame.tile_size =
            frame.data.len() - (frame.uncompressed_header_size + frame.compressed_header_size);

        self.refresh_ref_frames(&frame);

        Ok(frame)
    }

    // Implements spec "8.10 Reference frame update process".
    fn refresh_ref_frames(&mut self, frame: &Frame) {
        let flags = frame.refresh_frame_flags;
        self.ref_frame_sizes
            .iter_mut()
            .enumerate()
            .for_each(|(i, (width, height))| {
                if (flags >> i) & 1 == 1 {
                    *width = frame.width;
                    *height = frame.height;
                }
            });
    }

    fn frame_sync_code(&self, br: &mut BitReader) -> Result<()> {
        let frame_sync_byte_0 = br.read_u8(8)?;
        let frame_sync_byte_1 = br.read_u8(8)?;
        let frame_sync_byte_2 = br.read_u8(8)?;

        if frame_sync_byte_0 != 0x49 && frame_sync_byte_1 != 0x83 && frame_sync_byte_2 != 0x42 {
            return Err(Vp9ParserError::InvalidSyncByte);
        }

        Ok(())
    }

    fn color_config(&self, br: &mut BitReader, frame: &mut Frame) -> Result<()> {
        if frame.profile >= Profile::Profile2 {
            let ten_or_twelve_bit = br.read_bool()?;
            if ten_or_twelve_bit {
                frame.color_depth = ColorDepth::Depth12;
            } else {
                frame.color_depth = ColorDepth::Depth10;
            }
        };

        frame.color_space = br.read_u8(3)?.into();

        if frame.color_space == ColorSpace::Rgb {
            frame.color_range = ColorRange::FullSwing;
            if frame.profile == Profile::Profile1 || frame.profile == Profile::Profile3 {
                frame.subsampling_x = false;
                frame.subsampling_y = false;
                let _reserved_zero = br.read_u8(1)?;
            }
        } else {
            frame.color_range = br.read_bool()?.into();
            if frame.profile == Profile::Profile1 || frame.profile == Profile::Profile3 {
                frame.subsampling_x = br.read_bool()?;
                frame.subsampling_y = br.read_bool()?;
                let _reserved_zero = br.read_u8(1)?;
            }
        }

        Ok(())
    }

    fn frame_size(&self, br: &mut BitReader, frame: &mut Frame) -> Result<()> {
        let frame_width_minus_1 = br.read_u16(16)?;
        let frame_height_minus_1 = br.read_u16(16)?;
        frame.width = frame_width_minus_1 + 1;
        frame.height = frame_height_minus_1 + 1;

        self.compute_image_size(frame);

        Ok(())
    }

    fn render_size(&self, br: &mut BitReader, frame: &mut Frame) -> Result<()> {
        let render_and_frame_size_different = br.read_bool()?;
        if render_and_frame_size_different {
            let render_width_minus_1 = br.read_u16(16)?;
            let render_height_minus_1 = br.read_u16(16)?;
            frame.render_width = render_width_minus_1 + 1;
            frame.render_height = render_height_minus_1 + 1;
        } else {
            frame.render_width = frame.width;
            frame.render_height = frame.height;
        }

        Ok(())
    }

    fn frame_size_with_refs(&self, br: &mut BitReader, frame: &mut Frame) -> Result<()> {
        let mut found_ref = false;
        for i in 0..3 {
            found_ref = br.read_bool()?;
            if found_ref {
                let sizes = *self
                    .ref_frame_sizes
                    .get(frame.ref_frame_indices[i] as usize)
                    .ok_or(Vp9ParserError::InvalidRefFrameIndex)?;

                frame.width = sizes.0;
                frame.height = sizes.1;
                break;
            }
        }

        if !found_ref {
            self.frame_size(br, frame)?;
        } else {
            self.compute_image_size(frame);
        }

        self.render_size(br, frame)?;

        Ok(())
    }

    fn compute_image_size(&self, frame: &mut Frame) {
        frame.mi_cols = (frame.width + 7) >> 3;
        frame.mi_rows = (frame.height + 7) >> 3;
    }

    fn read_interpolation_filter(&self, br: &mut BitReader, frame: &mut Frame) -> Result<()> {
        let literal_to_type: [InterpolationFilter; 4] = [
            InterpolationFilter::EighttapSmooth,
            InterpolationFilter::Eighttap,
            InterpolationFilter::EighttapSharp,
            InterpolationFilter::Bilinear,
        ];

        let is_filter_switchable = br.read_bool()?;
        if is_filter_switchable {
            frame.interpolation_filter = InterpolationFilter::Switchable;
        } else {
            let raw_interpolation_filter = br.read_u8(2)?;
            frame.interpolation_filter = literal_to_type[raw_interpolation_filter as usize]
        }

        Ok(())
    }

    fn loop_filter_params(&mut self, br: &mut BitReader, frame: &mut Frame) -> Result<()> {
        frame.loop_filter_level = br.read_u8(6)?;
        frame.loop_filter_sharpness = br.read_u8(3)?;
        frame.loop_filter_delta_enabled = br.read_bool()?;

        if frame.loop_filter_delta_enabled {
            let loop_filter_delta_update = br.read_bool()?;
            if loop_filter_delta_update {
                for delta in self.loop_filter_ref_deltas.iter_mut() {
                    let update_ref_delta = br.read_bool()?;
                    if update_ref_delta {
                        *delta = br.read_inverse_i8(6)?;
                    }
                }

                for mode in self.loop_filter_mode_deltas.iter_mut() {
                    let update_mode_delta = br.read_bool()?;
                    if update_mode_delta {
                        *mode = br.read_inverse_i8(6)?;
                    }
                }
            }
        }

        Ok(())
    }

    fn quantization_params(&self, br: &mut BitReader, frame: &mut Frame) -> Result<()> {
        frame.base_q_idx = br.read_u8(8)? as i32;
        frame.delta_q_y_dc = self.read_delta_q(br)?;
        frame.delta_q_uv_dc = self.read_delta_q(br)?;
        frame.delta_q_uv_ac = self.read_delta_q(br)?;
        frame.lossless = frame.base_q_idx == 0
            && frame.delta_q_y_dc == 0
            && frame.delta_q_uv_dc == 0
            && frame.delta_q_uv_ac == 0;

        Ok(())
    }

    fn read_delta_q(&self, br: &mut BitReader) -> Result<i32> {
        let delta_coded = br.read_bool()?;
        if delta_coded {
            let delta_q = br.read_inverse_i8(4)? as i32;
            Ok(delta_q)
        } else {
            Ok(0)
        }
    }

    fn segmentation_params(&self, br: &mut BitReader, frame: &mut Frame) -> Result<()> {
        frame.segmentation_enabled = br.read_bool()?;
        if frame.segmentation_enabled {
            frame.segmentation_update_map = br.read_bool()?;
            if frame.segmentation_update_map {
                for prob in frame.segment_tree_probs.iter_mut() {
                    *prob = self.read_prob(br)?;
                }

                frame.segmentation_temporal_update = br.read_bool()?;
                for prob in frame.segment_pred_probs.iter_mut() {
                    *prob = if frame.segmentation_temporal_update {
                        self.read_prob(br)?
                    } else {
                        255
                    };
                }
            }

            frame.segmentation_update_data = br.read_bool()?;
            if frame.segmentation_update_data {
                frame.segmentation_abs_or_delta_update = br.read_bool()?;
                for i in 0..MAX_SEGMENTS {
                    frame.segment_feature_active[i].alt_q = br.read_bool()?;
                    if frame.segment_feature_active[i].alt_q {
                        frame.segment_feature_data[i].alt_q = br.read_inverse_i16(8)? as i16;
                    };
                    frame.segment_feature_active[i].alt_l = br.read_bool()?;
                    if frame.segment_feature_active[i].alt_l {
                        frame.segment_feature_data[i].alt_l = br.read_inverse_i16(6)? as i16;
                    };
                    frame.segment_feature_active[i].ref_frame = br.read_bool()?;
                    if frame.segment_feature_active[i].ref_frame {
                        frame.segment_feature_data[i].ref_frame = br.read_inverse_i16(2)? as i16;
                    };
                    frame.segment_feature_active[i].skip_segment = br.read_bool()?;
                }
            }
        }

        Ok(())
    }

    fn read_prob(&self, br: &mut BitReader) -> Result<u8> {
        let prob_coded = br.read_bool()?;
        if prob_coded {
            let prob = br.read_u8(8)?;
            Ok(prob)
        } else {
            Ok(255)
        }
    }

    fn tile_info(&self, br: &mut BitReader, frame: &mut Frame) -> Result<()> {
        let min_log2_tile_cols = self.calc_min_log2_tile_cols(frame);
        let max_log2_tile_cols = self.calc_max_log2_tile_cols(frame);
        frame.tile_rows_log2 = min_log2_tile_cols;
        while frame.tile_rows_log2 < max_log2_tile_cols {
            let increment_tile_cols_log2 = br.read_bool()?;
            if increment_tile_cols_log2 {
                frame.tile_cols_log2 += 1;
            } else {
                break;
            }
        }
        frame.tile_rows_log2 = br.read_u8(1)?;
        if frame.tile_rows_log2 == 1 {
            let increment_tile_rows_log2 = br.read_u8(1)?;
            frame.tile_rows_log2 += increment_tile_rows_log2;
        }

        Ok(())
    }

    fn calc_min_log2_tile_cols(&self, frame: &Frame) -> u8 {
        let mut min_log2 = 0;
        let sb64_cols = ((frame.mi_cols + 7) >> 3) as u8;
        while (64 << min_log2) < sb64_cols {
            min_log2 += 1;
        }
        min_log2
    }

    fn calc_max_log2_tile_cols(&self, frame: &Frame) -> u8 {
        let mut max_log2 = 1;
        let sb64_cols = ((frame.mi_cols + 7) >> 3) as u8;
        while (sb64_cols >> max_log2) >= 4 {
            max_log2 += 1;
        }
        max_log2 - 1
    }

    // Aligns the reader to the next byte offset.
    fn trailing_bits(&self, br: &mut BitReader) -> Result<()> {
        while br.is_aligned(1) {
            let zero_bit = br.read_bool()?;
            if zero_bit {
                return Err(Vp9ParserError::InvalidPadding);
            }
        }

        Ok(())
    }
}

// The sign bit is at the start and not the end (even though it's BE).
trait SignedRead {
    fn read_inverse_i8(&mut self, bits: u8) -> Result<i8>;
    fn read_inverse_i16(&mut self, bits: u8) -> Result<i16>;
}

impl<'a> SignedRead for BitReader<'a> {
    fn read_inverse_i8(&mut self, bits: u8) -> Result<i8> {
        debug_assert!(bits < 8);

        let value = self.read_u8(bits)?;
        if self.read_bool()? {
            Ok(-(value as i8))
        } else {
            Ok(value as i8)
        }
    }

    fn read_inverse_i16(&mut self, bits: u8) -> Result<i16> {
        debug_assert!(bits < 16);

        let value = self.read_u16(bits)?;
        if self.read_bool()? {
            Ok(-(value as i16))
        } else {
            Ok(value as i16)
        }
    }
}
