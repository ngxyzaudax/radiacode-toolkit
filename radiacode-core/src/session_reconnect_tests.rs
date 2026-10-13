use radiacode_protocol::{
    Command, DeviceVersions, Error as ProtocolError, FirmwareVersion, VirtString,
};

use crate::device::RadiaCode;
use crate::error::Error;
use crate::session_restore::SessionRestore;
use crate::test_support::{DeviceHandle, ScriptedReply, VirtualDevice, open_device};

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
