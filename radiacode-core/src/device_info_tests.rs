use radiacode_protocol::{Command, VirtString};

use crate::test_support::fixtures::spectrum_device;
use crate::test_support::{ScriptedReply, VirtualDevice, open_device};

#[tokio::test(start_paused = true)]
async fn fw_version_retries_transient_decode_failures() {
    let (mut device, handle) = open_device(VirtualDevice::default()).await;
    handle
        .device()
        .push_script(Command::GetVersion, ScriptedReply::Payload(vec![1, 2]));
    let versions = device.fw_version().await.expect("versions");
    assert_eq!((versions.target.major, versions.target.minor), (4, 12));
    assert_eq!((versions.boot.major, versions.boot.minor), (4, 0));
    assert_eq!(handle.device().request_count(Command::GetVersion), 3);
}

#[tokio::test(start_paused = true)]
async fn fw_version_gives_up_after_three_transient_failures() {
    let (mut device, handle) = open_device(VirtualDevice::default()).await;
    for _ in 0..3 {
        handle
            .device()
            .push_script(Command::GetVersion, ScriptedReply::Payload(vec![1]));
    }
    let error = device.fw_version().await.expect_err("exhausted");
    assert!(error.is_transient());
}

#[tokio::test(start_paused = true)]
async fn hw_serial_number_formats_groups() {
    let (mut device, _handle) = open_device(VirtualDevice::default()).await;
    let serial = device.hw_serial_number().await.expect("serial");
    assert_eq!(serial, "01020304-05060708");
}

#[tokio::test(start_paused = true)]
async fn serial_number_and_configuration_strings() {
    let device = VirtualDevice::default().with_string(VirtString::Configuration, &[0xC0, 0xC1]);
    let (mut device, _handle) = open_device(device).await;
    assert_eq!(device.serial_number().await.unwrap(), "RC-103-001234");
    assert_eq!(device.configuration().await.unwrap(), "АБ");
}

#[tokio::test(start_paused = true)]
async fn spectra_decode_with_session_format_version() {
    let (mut device, _handle) = open_device(spectrum_device()).await;
    let spectrum = device.spectrum().await.expect("spectrum");
    assert_eq!(spectrum.duration.as_secs(), 120);
    assert_eq!((spectrum.a0, spectrum.a1, spectrum.a2), (1.0, 2.0, 3.0));
    assert_eq!(spectrum.counts, vec![5, 6, 7]);
    assert_eq!(device.spectrum_accum().await.unwrap().counts, vec![5, 6, 7]);
    assert_eq!(device.spectrum_diff().await.unwrap().counts, vec![5, 6, 7]);
}
