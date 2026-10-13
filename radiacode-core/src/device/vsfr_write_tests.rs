use radiacode_protocol::{Command, Error as ProtocolError, VirtSfr};

use crate::test_support::encode::u32_words;
use crate::test_support::fixtures::protocol_error;
use crate::test_support::{ScriptedReply, VirtualDevice, open_device};

#[tokio::test(start_paused = true)]
async fn write_vsfr_stores_value() {
    let (mut device, handle) = open_device(VirtualDevice::default()).await;
    device
        .write_vsfr(VirtSfr::DispBrt, &5u32.to_le_bytes())
        .await
        .expect("write");
    assert_eq!(handle.device().register(VirtSfr::DispBrt), Some(5));
}

#[tokio::test(start_paused = true)]
async fn write_vsfr_optional_reports_rejection() {
    let (mut device, handle) = open_device(VirtualDevice::default()).await;
    handle
        .device()
        .push_script(Command::WrVirtSfr, ScriptedReply::Payload(u32_words(&[0])));
    handle
        .device()
        .push_script(Command::WrVirtSfr, ScriptedReply::Payload(u32_words(&[0])));
    let accepted = device
        .write_vsfr_optional(VirtSfr::DispBrt, &[1, 0, 0, 0])
        .await
        .expect("optional write");
    assert!(!accepted);
    let error = device
        .write_vsfr(VirtSfr::DispBrt, &[1, 0, 0, 0])
        .await
        .expect_err("rejected");
    assert!(matches!(
        protocol_error(error),
        ProtocolError::UnexpectedReturnCode(0)
    ));
}

#[tokio::test(start_paused = true)]
async fn write_vsfr_rejects_trailing_bytes_and_unknown_retcode() {
    let (mut device, handle) = open_device(VirtualDevice::default()).await;
    handle.device().push_script(
        Command::WrVirtSfr,
        ScriptedReply::Payload(u32_words(&[1, 9])),
    );
    handle
        .device()
        .push_script(Command::WrVirtSfr, ScriptedReply::Payload(u32_words(&[5])));
    let trailing = device
        .write_vsfr(VirtSfr::DispBrt, &[])
        .await
        .expect_err("trailing");
    assert!(matches!(
        protocol_error(trailing),
        ProtocolError::ProtocolMismatch { .. }
    ));
    let unknown = device
        .write_vsfr(VirtSfr::DispBrt, &[])
        .await
        .expect_err("retcode");
    assert!(matches!(
        protocol_error(unknown),
        ProtocolError::UnexpectedReturnCode(5)
    ));
}
