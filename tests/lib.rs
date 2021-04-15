use std::fs::File;

use vp9_parser::{parse_vp9_chunk, Ivf, IvfChunk};

#[test]
pub fn parse_ivf() {
    let file = File::open("tests/data/320-24-crf.ivf").unwrap();
    let ivf = Ivf::new(file).unwrap();

    assert_eq!(ivf.width(), 320);
    assert_eq!(ivf.height(), 180);
    assert_eq!(ivf.frame_rate_rate(), 24);
    assert_eq!(ivf.frame_rate_scale(), 1);
    assert_eq!(ivf.chunk_count(), 24);

    let mut first = true;
    let count: usize = ivf
        .iter()
        .map(|frame| {
            if first {
                assert_eq!(frame.timestamp, 0);
                first = false;
            } else {
                assert_ne!(frame.timestamp, 0);
            }

            assert_ne!(frame.data.len(), 0);
            1
        })
        .sum();
    assert_eq!(count, 24);
}

#[test]
pub fn parse_vp9_chunks() {
    // TODO Create a test file with super frames!
    let file = File::open("tests/data/320-24-crf.ivf").unwrap();
    let ivf = Ivf::new(file).unwrap();

    for chunk in ivf.iter() {
        let IvfChunk {
            timestamp: _timestamp,
            data,
        } = chunk;

        let _ = parse_vp9_chunk(data);
    }
}
