use radiacode_protocol::{Command, DecodeWarningKind, decode_data_buf};

use crate::data_buf_cursor::DataBufCursor;

use crate::test_support::encode::{rare_record, real_time_record};
use crate::test_support::fixtures::{device_with_frame, frame_with_rare, zero_alarm_limits};
use crate::test_support::open_device;

#[test]
fn disjoint_warning_counts() {
    let mut bytes = real_time_record(1, 0, 5.0, 0.001);
    bytes.extend(real_time_record(3, 0, 5.0, 0.001));
    let frame = decode_data_buf(&bytes);
    let rejected = frame
        .warnings
        .iter()
        .filter(|warning| matches!(warning.kind, DecodeWarningKind::SanityRejected(_)))
        .count();
    let seq_gaps = DataBufCursor::default().observe_frame(&frame);
    assert_eq!(rejected, 0);
    assert_eq!(seq_gaps.len(), 1);
}

#[tokio::test(start_paused = true)]
async fn poll_monitor_converts_records_and_uses_rare_status() {
    let (mut device, handle) = open_device(device_with_frame(&frame_with_rare())).await;
    let mut cursor = DataBufCursor::default();
    let (sample, status) = device
        .poll_monitor(&zero_alarm_limits(), &mut cursor, false)
        .await
        .expect("poll");
    assert_eq!(sample.rates.len(), 2);
    assert_eq!(sample.rates[0].count_rate, 300.0);
    assert!((sample.rates[0].dose_rate - 1.0).abs() < 1e-4);
    assert_eq!(sample.rates[0].count_rate_err_pct, 5.0);
    assert_eq!(sample.rates[0].dose_rate_err_pct, 7.0);
    let accumulated = sample.accumulated.expect("rare record");
    assert_eq!(accumulated.duration_secs, 3600);
    assert!((accumulated.dose - 25.0).abs() < 1e-3);
    assert_eq!(status.battery_percent, Some(87.5));
    assert_eq!(status.temperature_c, Some(24.5));
    assert_eq!(status.rssi_dbm, Some(-60));
    assert_eq!(sample.decode_warnings, 0);
    assert_eq!(sample.rejected_records, 0);
    assert_eq!(handle.device().request_count(Command::RdVirtSfr), 0);
}

#[tokio::test(start_paused = true)]
async fn poll_monitor_reads_temperature_register_without_rare_record() {
    let frame = real_time_record(1, 100, 5.0, 0.0001);
    let (mut device, handle) = open_device(device_with_frame(&frame)).await;
    let mut cursor = DataBufCursor::default();
    let (sample, status) = device
        .poll_monitor(&zero_alarm_limits(), &mut cursor, true)
        .await
        .expect("poll");
    assert!(sample.accumulated.is_none());
    assert_eq!(status.battery_percent, None);
    assert_eq!(status.temperature_c, Some(31.0));
    assert_eq!(status.rssi_dbm, Some(-61));
    assert_eq!(handle.device().request_count(Command::RdVirtSfr), 1);
}

#[tokio::test(start_paused = true)]
async fn poll_monitor_rejects_out_of_range_rare_values() {
    let mut frame = real_time_record(1, 100, 5.0, 0.0001);
    frame.extend(rare_record(2, 200, 160.0, 24.5));
    let (mut device, _handle) = open_device(device_with_frame(&frame)).await;
    let mut cursor = DataBufCursor::default();
    let (sample, status) = device
        .poll_monitor(&zero_alarm_limits(), &mut cursor, false)
        .await
        .expect("poll");
    assert_eq!(sample.rejected_records, 1);
    assert_eq!(status.battery_percent, None);
    assert_eq!(status.temperature_c, Some(31.0));
}
