use radiacode_protocol::{DecodeWarningKind, decode_data_buf};

use crate::data_buf_cursor::DataBufCursor;

#[test]
fn disjoint_warning_counts() {
    let count_rate = 5.0f32.to_le_bytes();
    let dose_rate = 0.001f32.to_le_bytes();
    let mut bytes = vec![1u8, 0, 0, 0, 0, 0, 0];
    bytes.extend_from_slice(&count_rate);
    bytes.extend_from_slice(&dose_rate);
    bytes.extend_from_slice(&[0u8; 7]);
    bytes.extend_from_slice(&[3u8, 0, 0, 0, 0, 0, 0]);
    bytes.extend_from_slice(&count_rate);
    bytes.extend_from_slice(&dose_rate);
    bytes.extend_from_slice(&[0u8; 7]);
    let frame = decode_data_buf(&bytes);
    let rejected = frame
        .warnings
        .iter()
        .filter(|warning| matches!(warning.kind, DecodeWarningKind::SanityRejected(_)))
        .count();
    let decode_warnings = frame
        .warnings
        .iter()
        .filter(|warning| {
            !matches!(
                warning.kind,
                DecodeWarningKind::SeqJump { .. } | DecodeWarningKind::SanityRejected(_)
            )
        })
        .count();
    let seq_gaps = DataBufCursor::default().observe_frame(&frame);
    assert_eq!(decode_warnings, 0);
    assert_eq!(rejected, 0);
    assert_eq!(seq_gaps.len(), 1);
}
