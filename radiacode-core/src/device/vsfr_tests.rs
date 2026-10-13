use radiacode_protocol::{BytesBuffer, Command, Error as ProtocolError, VirtSfr, VirtString};

use super::vsfr::{parse_spectrum_format_version, trim_trailing_nul_if_needed};
use crate::test_support::encode::{u32_words, virt_string_payload};
use crate::test_support::fixtures::protocol_error;
use crate::test_support::{ScriptedReply, VirtualDevice, open_device};

#[tokio::test(start_paused = true)]
async fn read_virt_string_returns_payload() {
    let (mut device, _handle) =
        open_device(VirtualDevice::default().with_string(VirtString::TextMessage, b"hello")).await;
    let payload = device
        .read_virt_string(VirtString::TextMessage)
        .await
        .expect("payload");
    assert_eq!(payload.data(), b"hello");
}

#[tokio::test(start_paused = true)]
async fn read_virt_string_rejects_non_success_retcode() {
    let (mut device, _handle) = open_device(VirtualDevice::default()).await;
    let error = device
        .read_virt_string(VirtString::TextMessage)
        .await
        .expect_err("unknown string");
    assert!(matches!(
        protocol_error(error),
        ProtocolError::UnexpectedReturnCode(0)
    ));
}

#[tokio::test(start_paused = true)]
async fn read_virt_string_trims_single_trailing_nul() {
    let (mut device, handle) = open_device(VirtualDevice::default()).await;
    let mut payload = virt_string_payload(1, b"abc");
    payload.push(0);
    handle
        .device()
        .push_script(Command::RdVirtString, ScriptedReply::Payload(payload));
    let value = device
        .read_virt_string(VirtString::TextMessage)
        .await
        .expect("payload");
    assert_eq!(value.data(), b"abc");
}

#[tokio::test(start_paused = true)]
async fn read_virt_string_truncates_overlong_payload() {
    let (mut device, handle) = open_device(VirtualDevice::default()).await;
    let mut payload = u32_words(&[1, 3]);
    payload.extend_from_slice(b"abcdef");
    handle
        .device()
        .push_script(Command::RdVirtString, ScriptedReply::Payload(payload));
    let value = device
        .read_virt_string(VirtString::TextMessage)
        .await
        .expect("payload");
    assert_eq!(value.data(), b"abc");
}

#[tokio::test(start_paused = true)]
async fn read_virt_string_reports_underrun() {
    let (mut device, handle) = open_device(VirtualDevice::default()).await;
    let mut payload = u32_words(&[1, 13]);
    payload.extend_from_slice(b"short");
    handle
        .device()
        .push_script(Command::RdVirtString, ScriptedReply::Payload(payload));
    let error = device
        .read_virt_string(VirtString::TextMessage)
        .await
        .expect_err("underrun");
    assert!(matches!(
        protocol_error(error),
        ProtocolError::BufferUnderrun { need: 13, have: 5 }
    ));
}

#[tokio::test(start_paused = true)]
async fn vsfr_u32_and_optional_reads() {
    let (mut device, _handle) =
        open_device(VirtualDevice::default().with_register(VirtSfr::DispBrt, 7)).await;
    assert_eq!(device.read_vsfr_u32(VirtSfr::DispBrt).await.unwrap(), 7);
    assert_eq!(
        device.read_vsfr_optional(VirtSfr::DispBrt).await.unwrap(),
        Some(7)
    );
    assert_eq!(
        device.read_vsfr_optional(VirtSfr::LedsOn).await.unwrap(),
        None
    );
    let error = device
        .read_vsfr_u32(VirtSfr::LedsOn)
        .await
        .expect_err("missing");
    assert!(matches!(
        protocol_error(error),
        ProtocolError::UnexpectedReturnCode(0)
    ));
}

#[tokio::test(start_paused = true)]
async fn vsfr_optional_rejects_unknown_retcode() {
    let (mut device, handle) = open_device(VirtualDevice::default()).await;
    handle.device().push_script(
        Command::RdVirtSfr,
        ScriptedReply::Payload(u32_words(&[7, 1])),
    );
    let error = device
        .read_vsfr_optional(VirtSfr::DispBrt)
        .await
        .expect_err("retcode 7");
    assert!(matches!(
        protocol_error(error),
        ProtocolError::UnexpectedReturnCode(7)
    ));
}

#[tokio::test(start_paused = true)]
async fn read_vsfr_f32_reinterprets_bits() {
    let raw = 1.5f32.to_bits();
    let (mut device, _handle) =
        open_device(VirtualDevice::default().with_register(VirtSfr::TempDegC, raw)).await;
    assert_eq!(device.read_vsfr_f32(VirtSfr::TempDegC).await.unwrap(), 1.5);
}

#[test]
fn trailing_nul_trimmed_only_when_exactly_one_extra_byte() {
    let mut exact = BytesBuffer::new(b"abc\0".to_vec());
    trim_trailing_nul_if_needed(&mut exact, 3);
    assert_eq!(exact.data(), b"abc");
    let mut longer = BytesBuffer::new(b"abc\0\0".to_vec());
    trim_trailing_nul_if_needed(&mut longer, 3);
    assert_eq!(longer.data(), b"abc\0\0");
    let mut non_nul = BytesBuffer::new(b"abcd".to_vec());
    trim_trailing_nul_if_needed(&mut non_nul, 3);
    assert_eq!(non_nul.data(), b"abcd");
}

#[test]
fn spectrum_format_version_line_scan() {
    assert_eq!(
        parse_spectrum_format_version("A=1\nSpecFormatVersion=2\n"),
        2
    );
    assert_eq!(parse_spectrum_format_version("SpecFormatVersion=x\n"), 0);
    assert_eq!(parse_spectrum_format_version(""), 0);
}
