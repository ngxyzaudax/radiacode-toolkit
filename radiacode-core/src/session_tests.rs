use radiacode_protocol::{
    Command, DeviceVersions, Error as ProtocolError, FirmwareVersion, VirtSfr, VirtString,
};

use crate::device::RadiaCode;
use crate::error::Error;
use crate::session_restore::SessionRestore;
use crate::test_support::{
    DeviceHandle, ScriptedReply, VirtualDevice, open_device, try_open_device,
};

fn restore_fixture() -> SessionRestore {
    SessionRestore {
        versions: DeviceVersions {
            boot: FirmwareVersion {
                major: 4,
                minor: 0,
                date: "boot".into(),
            },
            target: FirmwareVersion {
                major: 4,
                minor: 9,
                date: "target".into(),
            },
        },
        spectrum_format_version: 1,
    }
}

#[tokio::test(start_paused = true)]
async fn open_initializes_session_and_caches_versions() {
    let (device, handle) = open_device(VirtualDevice::default()).await;
    assert_eq!(device.spectrum_format_version(), 1);
    assert_eq!(device.sfr_entries().len(), 2);
    let restore = device.session_restore().expect("restore");
    assert_eq!(restore.versions.target.major, 4);
    assert_eq!(restore.versions.target.minor, 12);
    let state = handle.device();
    assert_eq!(state.request_count(Command::SetExchange), 1);
    assert_eq!(state.request_count(Command::SetTime), 1);
    assert_eq!(state.register(VirtSfr::DeviceTime), Some(0));
    assert_eq!(state.request_count(Command::GetVersion), 1);
    drop(state);
    assert!(handle.drain_count() >= 4);
}

#[tokio::test(start_paused = true)]
async fn open_reads_sfr_file_exactly_once() {
    let (_device, handle) = open_device(VirtualDevice::default()).await;
    let state = handle.device();
    let sfr_reads = state
        .requests_for(Command::RdVirtString)
        .into_iter()
        .filter(|request| request.args == u32::from(VirtString::SfrFile).to_le_bytes())
        .count();
    assert_eq!(sfr_reads, 1);
}

#[tokio::test(start_paused = true)]
async fn open_rejects_old_firmware_and_disconnects() {
    let (outcome, handle) = try_open_device(VirtualDevice::default().with_firmware(4, 7)).await;
    assert!(matches!(
        outcome,
        Err(Error::IncompatibleFirmware { major: 4, minor: 7 })
    ));
    assert!(handle.is_disconnected());
}

#[tokio::test(start_paused = true)]
async fn open_can_ignore_firmware_check() {
    let handle = DeviceHandle::new(VirtualDevice::default().with_firmware(4, 7));
    let device = RadiaCode::open(handle.transport(None), true, None).await;
    assert!(device.is_ok());
}

#[tokio::test(start_paused = true)]
async fn open_retries_transient_failures_then_succeeds() {
    let device = VirtualDevice::default().with_script(
        Command::SetExchange,
        ScriptedReply::Error(ProtocolError::Timeout),
    );
    let (outcome, handle) = try_open_device(device).await;
    assert!(outcome.is_ok());
    assert_eq!(handle.device().request_count(Command::SetExchange), 2);
}

#[tokio::test(start_paused = true)]
async fn open_gives_up_after_repeated_transient_failures() {
    let mut device = VirtualDevice::default();
    for _ in 0..3 {
        device.push_script(
            Command::SetExchange,
            ScriptedReply::Error(ProtocolError::Timeout),
        );
    }
    let (outcome, handle) = try_open_device(device).await;
    assert!(matches!(
        outcome,
        Err(Error::Protocol(ProtocolError::Timeout))
    ));
    assert!(handle.is_disconnected());
    assert_eq!(handle.device().request_count(Command::SetExchange), 3);
}

#[tokio::test(start_paused = true)]
async fn open_falls_back_to_config_line_scan_for_spectrum_version() {
    let device = VirtualDevice::default().with_string(
        VirtString::Configuration,
        b"Something=1\r\nSpecFormatVersion=0\n",
    );
    let (device, _handle) = open_device(device).await;
    assert_eq!(device.spectrum_format_version(), 0);
}

#[tokio::test(start_paused = true)]
async fn reconnect_restores_cached_session_without_version_query() {
    let handle = DeviceHandle::new(VirtualDevice::default().with_firmware(4, 7));
    let restore = restore_fixture();
    let device = RadiaCode::open(handle.transport(None), false, Some(&restore))
        .await
        .expect("restore");
    assert_eq!(device.spectrum_format_version(), 1);
    assert_eq!(device.session_restore(), Some(restore));
    assert_eq!(handle.device().request_count(Command::GetVersion), 0);
}

#[tokio::test(start_paused = true)]
async fn reconnect_uses_fewer_attempts() {
    let mut device = VirtualDevice::default();
    for _ in 0..2 {
        device.push_script(
            Command::SetExchange,
            ScriptedReply::Error(ProtocolError::Timeout),
        );
    }
    let handle = DeviceHandle::new(device);
    let restore = restore_fixture();
    let outcome = RadiaCode::open(handle.transport(None), false, Some(&restore)).await;
    assert!(outcome.is_err());
    assert_eq!(handle.device().request_count(Command::SetExchange), 2);
}

#[tokio::test(start_paused = true)]
async fn mismatched_response_header_is_protocol_mismatch() {
    let (mut device, handle) = open_device(VirtualDevice::default()).await;
    handle.device().push_script(
        Command::GetStatus,
        ScriptedReply::RawResponse(vec![0xff, 0xff, 0, 0, 1, 0, 0, 0]),
    );
    let error = device.status().await.expect_err("mismatch");
    assert!(matches!(
        error,
        Error::Protocol(ProtocolError::ProtocolMismatch { .. })
    ));
}

#[tokio::test(start_paused = true)]
async fn disconnect_releases_transport() {
    let (device, handle) = open_device(VirtualDevice::default()).await;
    device.disconnect().await.expect("disconnect");
    assert!(handle.is_disconnected());
}

#[tokio::test(start_paused = true)]
async fn rssi_and_wall_clock_delegate() {
    let (device, _handle) = open_device(VirtualDevice::default()).await;
    assert_eq!(device.rssi_dbm().await, Some(-60));
    assert_eq!(device.sample_rssi_dbm().await, Some(-61));
    let ticks = radiacode_protocol::DeviceTicks::new(100);
    let expected = device.base_time() + std::time::Duration::from_secs(1);
    assert_eq!(device.wall_clock(ticks), expected);
}
