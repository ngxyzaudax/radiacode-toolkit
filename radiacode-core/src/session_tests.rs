use radiacode_protocol::{Command, Error as ProtocolError, VirtSfr, VirtString};

use crate::device::RadiaCode;
use crate::error::Error;

use crate::test_support::{
    DeviceHandle, ScriptedReply, VirtualDevice, open_device, try_open_device,
};

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
