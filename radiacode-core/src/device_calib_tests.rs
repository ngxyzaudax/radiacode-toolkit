use radiacode_protocol::{Command, Error as ProtocolError, VirtSfr, VirtString};

use crate::device_language::DeviceLanguage;
use crate::error::Error;
use crate::test_support::encode::u32_words;
use crate::test_support::fixtures::spectrum_device;
use crate::test_support::{ScriptedReply, VirtualDevice, open_device};

#[tokio::test(start_paused = true)]
async fn energy_calib_read_and_write() {
    let (mut device, handle) = open_device(spectrum_device()).await;
    assert_eq!(device.energy_calib().await.unwrap(), [1.5, 2.5, 3.5]);
    device
        .set_energy_calib([0.5, 1.0, 0.0])
        .await
        .expect("write calib");
    let stored = handle.device().strings[&u32::from(VirtString::EnergyCalib)].clone();
    assert_eq!(
        stored,
        [
            0.5f32.to_le_bytes(),
            1.0f32.to_le_bytes(),
            0.0f32.to_le_bytes()
        ]
        .concat()
    );
}

#[tokio::test(start_paused = true)]
async fn set_energy_calib_rejects_non_success_retcode() {
    let (mut device, handle) = open_device(spectrum_device()).await;
    handle.device().push_script(
        Command::WrVirtString,
        ScriptedReply::Payload(u32_words(&[0])),
    );
    let error = device
        .set_energy_calib([0.0; 3])
        .await
        .expect_err("rejected");
    assert!(matches!(
        error,
        Error::Protocol(ProtocolError::UnexpectedReturnCode(0))
    ));
}

#[tokio::test(start_paused = true)]
async fn metadata_combines_serial_model_versions_and_calibration() {
    let (mut device, _handle) = open_device(spectrum_device()).await;
    let metadata = device.metadata().await.expect("metadata");
    assert_eq!(metadata.serial, "RC-103-001234");
    assert_eq!(metadata.model, "RC-103");
    assert_eq!(metadata.energy_calib, [1.5, 2.5, 3.5]);
    assert_eq!(metadata.firmware_label(), "4.12 (Feb 02 2024)");
}

#[tokio::test(start_paused = true)]
async fn diagnostics_commands_decode_payloads() {
    let device = VirtualDevice::default().with_string(VirtString::TextMessage, b"msg");
    let (mut device, _handle) = open_device(device).await;
    assert_eq!(device.status().await.unwrap(), 0x1234);
    let signature = device.fw_signature().await.unwrap();
    assert_eq!(signature.signature, 0xCAFE);
    assert_eq!(signature.filename, "rc-103.fw");
    assert_eq!(signature.id_string, "RadiaCode");
    assert_eq!(device.text_message().await.unwrap(), "msg");
    assert!(
        device
            .sfr_file_text()
            .await
            .unwrap()
            .contains("VSFR_DISP_BRT")
    );
}

#[tokio::test(start_paused = true)]
async fn settings_write_expected_registers() {
    let (mut device, handle) = open_device(VirtualDevice::default()).await;
    device.set_sound_on(true).await.unwrap();
    device.set_vibro_on(false).await.unwrap();
    device.set_display_brightness(9).await.unwrap();
    device.set_language(DeviceLanguage::English).await.unwrap();
    device.set_device_on(false).await.unwrap();
    device.dose_reset().await.unwrap();
    let state = handle.device();
    assert_eq!(state.register(VirtSfr::SoundOn), Some(1));
    assert_eq!(state.register(VirtSfr::VibroOn), Some(0));
    assert_eq!(state.register(VirtSfr::DispBrt), Some(9));
    assert_eq!(state.register(VirtSfr::DeviceLang), Some(1));
    assert_eq!(state.register(VirtSfr::DeviceOn), Some(0));
    assert_eq!(state.register(VirtSfr::DoseReset), Some(0));
}

#[tokio::test(start_paused = true)]
async fn brightness_above_nine_is_rejected_locally() {
    let (mut device, handle) = open_device(VirtualDevice::default()).await;
    let writes = handle.device().request_count(Command::WrVirtSfr);
    assert!(device.set_display_brightness(10).await.is_err());
    assert_eq!(handle.device().request_count(Command::WrVirtSfr), writes);
}

#[tokio::test(start_paused = true)]
async fn spectrum_reset_writes_empty_spectrum_string() {
    let (mut device, handle) = open_device(spectrum_device()).await;
    device.spectrum_reset().await.expect("reset");
    let first_write_args = handle.device().requests_for(Command::WrVirtString)[0]
        .args
        .clone();
    assert_eq!(
        first_write_args,
        u32_words(&[u32::from(VirtString::Spectrum), 0])
    );
    handle.device().push_script(
        Command::WrVirtString,
        ScriptedReply::Payload(u32_words(&[2])),
    );
    assert!(device.spectrum_reset().await.is_err());
}

#[test]
fn device_language_flags() {
    assert_eq!(DeviceLanguage::Russian.to_device_flag(), 0);
    assert_eq!(DeviceLanguage::English.to_device_flag(), 1);
}
