use radiacode_protocol::CountDisplayUnit;

use crate::error::Error;

use crate::test_support::fixtures::{device_with_frame, frame_with_rare, zero_alarm_limits};
use crate::test_support::open_device;

#[tokio::test(start_paused = true)]
async fn live_rates_use_latest_real_time_record() {
    let (mut device, _handle) = open_device(device_with_frame(&frame_with_rare())).await;
    let rates = device
        .live_rates(&zero_alarm_limits())
        .await
        .expect("rates");
    assert_eq!(rates.count_rate, 360.0);
    assert!((rates.dose_rate - 2.0).abs() < 1e-4);
    assert_eq!(rates.count_unit, CountDisplayUnit::Cpm);
}

#[tokio::test(start_paused = true)]
async fn live_rates_report_pending_when_buffer_has_no_rates() {
    let (mut device, _handle) = open_device(device_with_frame(&[])).await;
    let error = device
        .live_rates(&zero_alarm_limits())
        .await
        .expect_err("pending");
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
