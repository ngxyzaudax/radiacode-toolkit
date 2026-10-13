use super::{DataBufRecord, DecodeWarningKind, decode_data_buf};

#[test]
fn unknown_record_stops_decode_without_phantom_seq_jumps() {
    let count_rate = 5.0f32.to_le_bytes();
    let dose_rate = 0.001f32.to_le_bytes();
    let mut bytes = vec![1u8, 99, 99, 0, 0, 0, 0];
    bytes.extend_from_slice(&[0u8; 8]);
    bytes.extend_from_slice(&[2u8, 0, 0, 0, 0, 0, 0]);
    bytes.extend_from_slice(&count_rate);
    bytes.extend_from_slice(&dose_rate);
    bytes.extend_from_slice(&[0u8; 7]);
    let frame = decode_data_buf(&bytes);
    assert_eq!(
        frame
            .warnings
            .iter()
            .filter(|warning| matches!(warning.kind, DecodeWarningKind::UnknownRecord { .. }))
            .count(),
        1
    );
    assert!(
        !frame
            .warnings
            .iter()
            .any(|warning| matches!(warning.kind, DecodeWarningKind::SeqJump { .. }))
    );
    assert_eq!(frame.records.len(), 0);
}

fn build_event_record(seq: u8, event: u8, param: u8, flags: u16) -> Vec<u8> {
    let mut bytes = vec![seq, 0, 7, 0x00, 0x00, 0x00, 0x00];
    bytes.push(event);
    bytes.push(param);
    bytes.extend_from_slice(&flags.to_le_bytes());
    bytes
}

#[test]
fn parses_event_record_with_four_byte_payload() {
    let bytes = build_event_record(1, 0x14, 0x03, 0x1141);
    let frame = decode_data_buf(&bytes);
    assert_eq!(frame.warnings.len(), 0);
    assert_eq!(frame.records.len(), 1);
    let DataBufRecord::Event(record) = &frame.records[0] else {
        panic!("expected event record");
    };
    assert_eq!(record.event_param1, 0x03);
    assert_eq!(record.flags.raw(), 0x1141);
}

#[test]
fn event_followed_by_real_time_decodes_both_without_resync() {
    let count_rate = 12.5f32.to_le_bytes();
    let dose_rate = 0.000_125f32.to_le_bytes();
    let mut bytes = build_event_record(1, 0x04, 0x00, 0x0000);
    bytes.extend_from_slice(&[2u8, 0, 0, 0, 0, 0, 0]);
    bytes.extend_from_slice(&count_rate);
    bytes.extend_from_slice(&dose_rate);
    bytes.extend_from_slice(&[0u8; 7]);
    let frame = decode_data_buf(&bytes);
    assert_eq!(frame.records.len(), 2);
    assert_eq!(frame.resync_count, 0);
    assert!(matches!(&frame.records[0], DataBufRecord::Event(_)));
    assert!(matches!(&frame.records[1], DataBufRecord::RealTime(_)));
    assert!(!frame.warnings.iter().any(|warning| matches!(
        warning.kind,
        DecodeWarningKind::SeqJump { .. }
            | DecodeWarningKind::UnknownRecord { .. }
            | DecodeWarningKind::TruncatedTail
    )));
}

#[test]
fn resyncs_after_garbage_without_phantom_seq_jumps() {
    let count_rate = 5.0f32.to_le_bytes();
    let dose_rate = 0.001f32.to_le_bytes();
    let mut bytes = vec![1u8, 0, 0, 0, 0, 0, 0];
    bytes.extend_from_slice(&count_rate);
    bytes.extend_from_slice(&dose_rate);
    bytes.extend_from_slice(&[0u8; 7]);
    bytes.extend_from_slice(&[2, 99, 99, 0, 0, 0, 0]);
    bytes.extend_from_slice(&[0xAA, 0xBB, 0xCC]);
    bytes.extend_from_slice(&[2u8, 0, 0, 0, 0, 0, 0]);
    bytes.extend_from_slice(&count_rate);
    bytes.extend_from_slice(&dose_rate);
    bytes.extend_from_slice(&[0u8; 7]);
    let frame = decode_data_buf(&bytes);
    assert_eq!(frame.records.len(), 2);
    assert!(frame.resync_count >= 1);
    assert!(
        !frame
            .warnings
            .iter()
            .any(|warning| matches!(warning.kind, DecodeWarningKind::SeqJump { .. }))
    );
}
