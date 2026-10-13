use super::{
    DataBufRecord, DecodeWarningKind, DeviceTicks, EventId, RecordKind, Seq, decode_data_buf,
    latest_rare_record, latest_snapshot, real_time_records,
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
fn record_kind_mapping_covers_known_groups() {
    assert_eq!(RecordKind::from_entity_group(0, 2), RecordKind::DoseRateDb);
    assert_eq!(
        RecordKind::from_entity_group(0, 8),
        RecordKind::RawCountRate
    );
    assert_eq!(RecordKind::from_entity_group(1, 2), RecordKind::Waveform16);
    assert_eq!(
        RecordKind::from_entity_group(9, 9),
        RecordKind::Unknown {
            entity: 9,
            group: 9
        }
    );
    assert_eq!(RecordKind::RealTimeData.monitor_source_rank(), 3);
    assert_eq!(RecordKind::DoseRateDb.monitor_source_rank(), 2);
    assert_eq!(RecordKind::Event.monitor_source_rank(), 0);
}

#[test]
fn event_ids_map_known_values_and_keep_unknown_raw() {
    assert_eq!(EventId::from_raw(0), EventId::PowerOff);
    assert_eq!(EventId::from_raw(4), EventId::DoseReset);
    assert_eq!(EventId::from_raw(22), EventId::CountRateOffscale);
    assert_eq!(EventId::from_raw(200), EventId::Unknown(200));
}

#[test]
fn device_ticks_conversions() {
    let ticks = DeviceTicks::new(250);
    assert_eq!(ticks.raw(), 250);
    assert_eq!(ticks.as_duration().as_millis(), 2500);
    assert_eq!(DeviceTicks::new(-5).as_duration().as_millis(), 0);
    let base = std::time::UNIX_EPOCH;
    assert_eq!(
        ticks.wall_clock_time(base),
        base + std::time::Duration::from_millis(2500)
    );
}

#[test]
fn seq_loss_accounting() {
    assert_eq!(Seq::new(5).lost_since(Seq::new(5)), Some(0));
    assert_eq!(Seq::new(7).lost_since(Seq::new(5)), Some(2));
    assert_eq!(Seq::new(3).lost_since(Seq::new(5)), None);
    assert_eq!(Seq::new(255).next(), Seq::new(0));
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
