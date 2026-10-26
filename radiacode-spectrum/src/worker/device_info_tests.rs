use crate::test_device::{ScriptedReply, VirtualDevice};
use radiacode_lib::protocol::{Command, VirtString};
use radiacode_lib::{DeviceEndpoint, TransportKind};

use super::device_info::{load_device_info, load_device_info_with_retry};
use super::messages::WorkerEvent;
use super::test_fixtures::{Harness, configured_device, count_events, usb_endpoint};

fn calibrated_device() -> VirtualDevice {
    let calibration = [
        1.5f32.to_le_bytes(),
        2.5f32.to_le_bytes(),
        3.5f32.to_le_bytes(),
    ]
    .concat();
    configured_device().with_string(VirtString::EnergyCalib, &calibration)
}

fn bluetooth_endpoint() -> DeviceEndpoint {
    DeviceEndpoint::Bluetooth {
        address: "AA:BB:CC:DD:EE:FF".into(),
    }
}

fn is_alarm_limits(event: &WorkerEvent) -> bool {
    matches!(event, WorkerEvent::AlarmLimits(_))
}

#[tokio::test(start_paused = true)]
async fn device_info_combines_metadata_status_and_alarm_limits() {
    let mut harness = Harness::connected(calibrated_device()).await;
    let mut device = harness.session.device.take().expect("device");
    let info = load_device_info(&mut device, &usb_endpoint(), None, &harness.sender)
        .await
        .expect("device info");
    assert_eq!(info.serial, "RC-103-001234");
    assert_eq!(info.transport, TransportKind::Usb);
    assert_eq!(info.energy_calib, [1.5, 2.5, 3.5]);
    assert_eq!(info.rssi_dbm, Some(-60));
    assert_eq!(count_events(&harness.drain(), is_alarm_limits), 1);
}

#[tokio::test(start_paused = true)]
async fn bluetooth_info_samples_fresh_link_rssi() {
    let mut harness = Harness::connected(calibrated_device()).await;
    let mut device = harness.session.device.take().expect("device");
    let info = load_device_info(
        &mut device,
        &bluetooth_endpoint(),
        Some(-40),
        &harness.sender,
    )
    .await
    .expect("device info");
    assert_eq!(info.transport, TransportKind::Bluetooth);
    assert_eq!(info.rssi_dbm, Some(-61));
}

#[tokio::test(start_paused = true)]
async fn transient_metadata_failure_is_retried_once() {
    let mut harness = Harness::connected(calibrated_device()).await;
    harness.script(Command::RdVirtString, ScriptedReply::timeout());
    let mut device = harness.session.device.take().expect("device");
    let info = load_device_info_with_retry(&mut device, &usb_endpoint(), None, &harness.sender)
        .await
        .expect("device info after retry");
    assert_eq!(info.serial, "RC-103-001234");
}

#[tokio::test(start_paused = true)]
async fn persistent_metadata_failure_is_reported() {
    let mut harness = Harness::connected(calibrated_device()).await;
    harness.script(Command::RdVirtString, ScriptedReply::timeout());
    harness.script(Command::RdVirtString, ScriptedReply::timeout());
    let mut device = harness.session.device.take().expect("device");
    let error = load_device_info_with_retry(&mut device, &usb_endpoint(), None, &harness.sender)
        .await
        .expect_err("second failure propagates");
    assert!(error.is_timeout());
    assert!(harness.drain().is_empty());
}
