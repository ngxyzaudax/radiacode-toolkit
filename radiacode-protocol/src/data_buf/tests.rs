use super::{
    DecodeWarningKind, RecordKind, Seq, decode_data_buf, latest_real_time_rates, latest_snapshot,
};

#[test]
fn parses_rare_data_record() {
    let bytes = [
        1u8, 0, 3, 0, 0, 0, 0, 10, 0, 0, 0, 0, 0, 0, 0, 0xd4, 0x10, 0x34, 0x21, 0x00, 0x00,
    ];
    let snapshot = latest_snapshot(&bytes);
    let status = snapshot.rare.expect("rare");
    assert_eq!(status.duration_secs, 10);
    assert!((status.temperature_c - 23.08).abs() < 0.01);
    assert!((status.battery_percent - 85.0).abs() < 0.01);
}

#[test]
fn parses_real_time_data_record() {
    let count_rate = 12.5f32.to_le_bytes();
    let dose_rate = 0.000_125f32.to_le_bytes();
    let mut bytes = vec![1u8, 0, 0, 0, 0, 0, 0];
    bytes.extend_from_slice(&count_rate);
    bytes.extend_from_slice(&dose_rate);
    bytes.extend_from_slice(&[10u8, 0, 20, 0, 0, 0, 0]);
    let rates = latest_real_time_rates(&bytes).expect("rates");
    assert!((rates.count_rate_cps - 12.5).abs() < 0.001);
    assert!((rates.dose_rate_rh - 0.000_125).abs() < 1e-9);
    assert!((rates.count_rate_err_pct - 1.0).abs() < 0.001);
    assert!((rates.dose_rate_err_pct - 2.0).abs() < 0.001);
}

#[test]
fn decode_reports_seq_jump() {
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
    assert!(frame.warnings.iter().any(|warning| matches!(
        warning.kind,
        DecodeWarningKind::SeqJump {
            expected: 2,
            got: 3
        }
    )));
}

#[test]
fn rejects_non_finite_rates() {
    let count_rate = f32::NAN.to_le_bytes();
    let dose_rate = 0.001f32.to_le_bytes();
    let mut bytes = vec![1u8, 0, 0, 0, 0, 0, 0];
    bytes.extend_from_slice(&count_rate);
    bytes.extend_from_slice(&dose_rate);
    bytes.extend_from_slice(&[0u8; 7]);
    let frame = decode_data_buf(&bytes);
    assert!(frame.records.is_empty());
    assert!(
        frame
            .warnings
            .iter()
            .any(|warning| matches!(warning.kind, DecodeWarningKind::SanityRejected(_)))
    );
}

#[test]
fn seq_wrap_prefers_newer_record() {
    let older = Seq::new(250);
    let newer = Seq::new(2);
    assert!(newer.is_newer_than(older));
    let count_rate = 5.0f32.to_le_bytes();
    let dose_rate = 0.001f32.to_le_bytes();
    let mut bytes = vec![250u8, 0, 0, 0, 0, 0, 0];
    bytes.extend_from_slice(&count_rate);
    bytes.extend_from_slice(&dose_rate);
    bytes.extend_from_slice(&[0u8; 7]);
    bytes.extend_from_slice(&[2u8, 0, 1, 0, 0, 0, 0]);
    let high = 30.0f32.to_le_bytes();
    let low = 0.001f32.to_le_bytes();
    bytes.extend_from_slice(&high);
    bytes.extend_from_slice(&low);
    bytes.extend_from_slice(&[0u8; 7]);
    let snapshot = latest_snapshot(&bytes);
    let rates = snapshot.rates.expect("rates");
    assert!((rates.count_rate_cps - 30.0).abs() < 0.001);
    assert_eq!(RecordKind::RawData.monitor_source_rank(), 1);
}
