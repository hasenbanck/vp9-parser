use std::fs::File;

use vp9_parser::ivf::{Frame, Ivf};
use vp9_parser::parse_vp9_chunk;

#[test]
pub fn parse_ivf() {
    let file = File::open("tests/data/320-24-crf.ivf").unwrap();
    let mut ivf = Ivf::new(file).unwrap();

    assert_eq!(ivf.width(), 320);
    assert_eq!(ivf.height(), 180);
    assert_eq!(ivf.frame_rate_rate(), 24);
    assert_eq!(ivf.frame_rate_scale(), 1);
    assert_eq!(ivf.frame_count(), 24);

    let mut first = true;

    let mut count = 0;
    while let Some(frame) = ivf.read_frame().unwrap() {
        if first {
            assert_eq!(frame.timestamp, 0);
            first = false;
        } else {
            assert_ne!(frame.timestamp, 0);
        }

        assert_ne!(frame.data.len(), 0);
        count += 1;
    }

    assert_eq!(count, 24);
}

#[test]
pub fn parse_vp9_chunks() {
    // 320-24-cq.ivf contains super frames with reference frames.
    let file = File::open("tests/data/320-24-cq.ivf").unwrap();
    let mut ivf = Ivf::new(file).unwrap();

    while let Some(frame) = ivf.read_frame().unwrap() {
        let Frame {
            timestamp: _timestamp,
            data,
        } = frame;

        let _ = parse_vp9_chunk(data);

        // TODO Test the frames. Also test super-frames (reference frame)
    }
}
