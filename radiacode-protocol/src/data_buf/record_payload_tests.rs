use super::{
    DataBufRecord, DecodeWarningKind, RecordKind, decode_data_buf, latest_rare_record,
    latest_snapshot, real_time_records,
};

fn header(seq: u8, entity: u8, group: u8) -> Vec<u8> {
    let mut bytes = vec![seq, entity, group];
    bytes.extend_from_slice(&100i32.to_le_bytes());
    bytes
}

fn real_time(seq: u8, cps: f32) -> Vec<u8> {
    let mut bytes = header(seq, 0, 0);
    bytes.extend_from_slice(&cps.to_le_bytes());
    bytes.extend_from_slice(&0.001f32.to_le_bytes());
    bytes.extend_from_slice(&[0u8; 7]);
    bytes
}

#[test]
fn skipped_record_kinds_are_consumed_with_correct_lengths() {
    let mut bytes = header(1, 0, 4);
    bytes.extend_from_slice(&[0u8; 16]);
    bytes.extend(header(2, 0, 8));
    bytes.extend_from_slice(&[0u8; 6]);
    bytes.extend(header(3, 1, 1));
    bytes.extend_from_slice(&2u16.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&[0u8; 16]);
    bytes.extend(real_time(4, 3.0));
    let frame = decode_data_buf(&bytes);
    assert_eq!(frame.records.len(), 4);
    assert!(matches!(frame.records[0], DataBufRecord::Skipped(_)));
    assert_eq!(frame.records[2].header().kind, RecordKind::Waveform8);
    assert!(frame.warnings.is_empty());
    assert_eq!(real_time_records(&frame).count(), 1);
}

#[test]
fn raw_dose_rate_db_and_accel_records_decode() {
    let mut bytes = header(1, 0, 1);
    bytes.extend_from_slice(&2.0f32.to_le_bytes());
    bytes.extend_from_slice(&0.002f32.to_le_bytes());
    bytes.extend(header(2, 0, 2));
    bytes.extend_from_slice(&40u32.to_le_bytes());
    bytes.extend_from_slice(&4.0f32.to_le_bytes());
    bytes.extend_from_slice(&0.004f32.to_le_bytes());
    bytes.extend_from_slice(&15u16.to_le_bytes());
    bytes.extend_from_slice(&0u16.to_le_bytes());
    bytes.extend(header(3, 0, 6));
    bytes.extend_from_slice(&[1, 0, 2, 0, 3, 0]);
    let frame = decode_data_buf(&bytes);
    assert_eq!(frame.records.len(), 3);
    let DataBufRecord::DoseRateDb(db) = &frame.records[1] else {
        panic!("dose rate db");
    };
    assert_eq!(db.count, 40);
    assert_eq!(db.dose_rate_err_pct, 1.5);
    let DataBufRecord::Accel(accel) = &frame.records[2] else {
        panic!("accel");
    };
    assert_eq!((accel.x, accel.y, accel.z), (1, 2, 3));
    let snapshot = latest_snapshot(&bytes);
    assert_eq!(snapshot.rates.unwrap().count_rate_cps, 4.0);
}

#[test]
fn truncated_payload_is_reported() {
    let mut bytes = header(1, 0, 0);
    bytes.extend_from_slice(&[0u8; 3]);
    let frame = decode_data_buf(&bytes);
    assert!(
        frame
            .warnings
            .iter()
            .any(|w| matches!(w.kind, DecodeWarningKind::PayloadUnderrun))
    );
    let mut tail = real_time(1, 1.0);
    tail.extend_from_slice(&[1, 2, 3]);
    let frame = decode_data_buf(&tail);
    assert!(
        frame
            .warnings
            .iter()
            .any(|w| matches!(w.kind, DecodeWarningKind::TruncatedTail))
    );
}

#[test]
fn latest_rare_record_returns_last_one() {
    let mut bytes = Vec::new();
    for (seq, duration) in [(1u8, 10u32), (2, 20)] {
        bytes.extend(header(seq, 0, 3));
        bytes.extend_from_slice(&duration.to_le_bytes());
        bytes.extend_from_slice(&0.0f32.to_le_bytes());
        bytes.extend_from_slice(&4000u16.to_le_bytes());
        bytes.extend_from_slice(&5000u16.to_le_bytes());
        bytes.extend_from_slice(&0u16.to_le_bytes());
    }
    let frame = decode_data_buf(&bytes);
    assert_eq!(latest_rare_record(&frame).unwrap().duration_secs, 20);
    assert!(latest_rare_record(&decode_data_buf(&[])).is_none());
}

#[test]
fn resync_gives_up_after_limit() {
    let mut bytes = real_time(1, 1.0);
    for seq in 2..8u8 {
        bytes.extend(header(seq, 99, 99));
        bytes.extend_from_slice(&[0xAA]);
        bytes.extend(real_time(seq, 1.0));
    }
    let frame = decode_data_buf(&bytes);
    assert!(frame.resync_count <= 4);
    assert!(frame.records.len() <= 5);
}
