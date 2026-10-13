use radiacode_protocol::{
    Command, CountDisplayUnit, DecodeWarningKind, DoseDisplayUnit, VirtSfr, VirtString,
    decode_data_buf,
};

use crate::data_buf_cursor::DataBufCursor;
use crate::error::Error;
use crate::test_support::encode::{rare_record, real_time_record};
use crate::test_support::{VirtualDevice, open_device};
use crate::types::AlarmLimits;

fn units() -> AlarmLimits {
    AlarmLimits {
        l1_count_rate: 0.0,
        l2_count_rate: 0.0,
        l1_dose_rate: 0.0,
        l2_dose_rate: 0.0,
        l1_dose: 0.0,
        l2_dose: 0.0,
        dose_unit: DoseDisplayUnit::MicroSievertPerHour,
        count_unit: CountDisplayUnit::Cpm,
    }
}

fn frame_with_rare() -> Vec<u8> {
    let mut bytes = real_time_record(1, 100, 5.0, 0.0001);
    bytes.extend(rare_record(2, 200, 87.5, 24.5));
    bytes.extend(real_time_record(3, 300, 6.0, 0.0002));
    bytes
}

fn device_with_frame(frame: &[u8]) -> VirtualDevice {
    VirtualDevice::default()
        .with_string(VirtString::DataBuf, frame)
        .with_register(VirtSfr::TempDegC, 31.0f32.to_bits())
}

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
        .poll_monitor(&units(), &mut cursor, false)
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
        .poll_monitor(&units(), &mut cursor, true)
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
        .poll_monitor(&units(), &mut cursor, false)
        .await
        .expect("poll");
    assert_eq!(sample.rejected_records, 1);
    assert_eq!(status.battery_percent, None);
    assert_eq!(status.temperature_c, Some(31.0));
}

#[tokio::test(start_paused = true)]
async fn live_rates_use_latest_real_time_record() {
    let (mut device, _handle) = open_device(device_with_frame(&frame_with_rare())).await;
    let rates = device.live_rates(&units()).await.expect("rates");
    assert_eq!(rates.count_rate, 360.0);
    assert!((rates.dose_rate - 2.0).abs() < 1e-4);
    assert_eq!(rates.count_unit, CountDisplayUnit::Cpm);
}

#[tokio::test(start_paused = true)]
async fn live_rates_report_pending_when_buffer_has_no_rates() {
    let (mut device, _handle) = open_device(device_with_frame(&[])).await;
    let error = device.live_rates(&units()).await.expect_err("pending");
    assert!(matches!(error, Error::MonitorDataPending));
}

#[tokio::test(start_paused = true)]
async fn device_status_reads_rare_record_from_buffer() {
    let (mut device, _handle) = open_device(device_with_frame(&frame_with_rare())).await;
    let status = device.device_status(false).await.expect("status");
    assert_eq!(status.battery_percent, Some(87.5));
    assert_eq!(status.temperature_c, Some(24.5));
}

#[test]
fn merge_status_keeps_previous_values_when_fresh_is_missing() {
    use crate::status_read::merge_status;
    use crate::types::DeviceStatus;
    let mut carry = DeviceStatus {
        battery_percent: Some(50.0),
        temperature_c: Some(20.0),
        rssi_dbm: Some(-70),
    };
    merge_status(
        &mut carry,
        DeviceStatus {
            battery_percent: None,
            temperature_c: Some(21.0),
            rssi_dbm: None,
        },
    );
    assert_eq!(carry.battery_percent, Some(50.0));
    assert_eq!(carry.temperature_c, Some(21.0));
    assert_eq!(carry.rssi_dbm, Some(-70));
}
