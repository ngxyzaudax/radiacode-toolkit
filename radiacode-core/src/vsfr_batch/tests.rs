use radiacode_protocol::{Command, Error as ProtocolError, VirtSfr};

use crate::error::Error;
use crate::test_support::encode::u32_words;
use crate::test_support::{ScriptedReply, VirtualDevice, open_device};
use crate::vsfr_batch::{read_vsfr_batch, read_vsfr_batch_raw, write_vsfr_batch};

const IDS: [VirtSfr; 3] = [VirtSfr::DispBrt, VirtSfr::SoundOn, VirtSfr::VibroOn];

fn populated() -> VirtualDevice {
    VirtualDevice::default()
        .with_register(VirtSfr::DispBrt, 0x0700_0000)
        .with_register(VirtSfr::SoundOn, 1)
        .with_register(VirtSfr::VibroOn, 0)
}

#[tokio::test(start_paused = true)]
async fn full_batch_read_extracts_sub_word_registers() {
    let (mut device, handle) = open_device(populated()).await;
    let values = read_vsfr_batch(&mut device, &IDS).await.expect("batch");
    assert_eq!(values, vec![7, 1, 0]);
    assert_eq!(handle.device().request_count(Command::RdVirtSfrBatch), 1);
    assert_eq!(handle.device().request_count(Command::RdVirtSfr), 0);
}

#[tokio::test(start_paused = true)]
async fn raw_batch_read_skips_extraction() {
    let (mut device, _handle) = open_device(populated()).await;
    let values = read_vsfr_batch_raw(&mut device, &IDS).await.expect("raw");
    assert_eq!(values, vec![0x0700_0000, 1, 0]);
}

#[tokio::test(start_paused = true)]
async fn sparse_batch_fills_missing_registers_individually() {
    let device = populated().with_register(VirtSfr::DispOffTime, 3);
    let (mut device, handle) = open_device(device).await;
    handle
        .device()
        .registers
        .remove(&u32::from(VirtSfr::SoundOn));
    handle
        .device()
        .registers
        .insert(u32::from(VirtSfr::SoundOn), 1);
    handle.device().push_script(
        Command::RdVirtSfrBatch,
        ScriptedReply::Payload(u32_words(&[0b101, 0x0700_0000, 0])),
    );
    let values = read_vsfr_batch(&mut device, &IDS).await.expect("sparse");
    assert_eq!(values, vec![7, 1, 0]);
    assert_eq!(handle.device().request_count(Command::RdVirtSfr), 1);
}

#[tokio::test(start_paused = true)]
async fn empty_valid_flags_is_reported_without_fallback() {
    let (mut device, handle) = open_device(populated().without_batch()).await;
    let error = read_vsfr_batch(&mut device, &IDS).await.expect_err("empty");
    assert!(matches!(
        error,
        Error::Protocol(ProtocolError::VsfrBatchEmpty)
    ));
    assert_eq!(handle.device().request_count(Command::RdVirtSfr), 0);
}

#[tokio::test(start_paused = true)]
async fn transient_batch_failures_retry_then_fall_back_to_sequential_reads() {
    let (mut device, handle) = open_device(populated()).await;
    for _ in 0..3 {
        handle.device().push_script(
            Command::RdVirtSfrBatch,
            ScriptedReply::Error(ProtocolError::Timeout),
        );
    }
    let values = read_vsfr_batch(&mut device, &IDS).await.expect("fallback");
    assert_eq!(values, vec![7, 1, 0]);
    assert_eq!(handle.device().request_count(Command::RdVirtSfrBatch), 3);
    assert_eq!(handle.device().request_count(Command::RdVirtSfr), 3);
}

#[tokio::test(start_paused = true)]
async fn non_transient_batch_failure_falls_back_immediately() {
    let (mut device, handle) = open_device(populated()).await;
    handle.device().push_script(
        Command::RdVirtSfrBatch,
        ScriptedReply::Error(ProtocolError::UnexpectedReturnCode(9)),
    );
    let values = read_vsfr_batch(&mut device, &IDS).await.expect("fallback");
    assert_eq!(values, vec![7, 1, 0]);
    assert_eq!(handle.device().request_count(Command::RdVirtSfrBatch), 1);
}

#[tokio::test(start_paused = true)]
async fn batch_size_is_validated() {
    let (mut device, _handle) = open_device(populated()).await;
    assert!(read_vsfr_batch(&mut device, &[]).await.is_err());
    assert!(write_vsfr_batch(&mut device, &[]).await.is_err());
    let too_many = vec![VirtSfr::DispBrt; 33];
    assert!(read_vsfr_batch(&mut device, &too_many).await.is_err());
}

#[tokio::test(start_paused = true)]
async fn write_batch_stores_values_and_validates_mask() {
    let (mut device, handle) = open_device(VirtualDevice::default()).await;
    write_vsfr_batch(&mut device, &[(VirtSfr::DispBrt, 4), (VirtSfr::SoundOn, 1)])
        .await
        .expect("write");
    assert_eq!(handle.device().register(VirtSfr::DispBrt), Some(4));
    assert_eq!(handle.device().register(VirtSfr::SoundOn), Some(1));
    for _ in 0..3 {
        handle.device().push_script(
            Command::WrVirtSfrBatch,
            ScriptedReply::Payload(u32_words(&[0b01])),
        );
    }
    let error = write_vsfr_batch(&mut device, &[(VirtSfr::DispBrt, 4), (VirtSfr::SoundOn, 1)])
        .await
        .expect_err("partial mask");
    assert!(matches!(
        error,
        Error::Protocol(ProtocolError::ProtocolMismatch { .. })
    ));
}

#[tokio::test(start_paused = true)]
async fn write_batch_retries_transient_errors() {
    let (mut device, handle) = open_device(VirtualDevice::default()).await;
    handle.device().push_script(
        Command::WrVirtSfrBatch,
        ScriptedReply::Error(ProtocolError::Timeout),
    );
    write_vsfr_batch(&mut device, &[(VirtSfr::DispBrt, 2)])
        .await
        .expect("retried");
    assert_eq!(handle.device().request_count(Command::WrVirtSfrBatch), 2);
    for _ in 0..3 {
        handle.device().push_script(
            Command::WrVirtSfrBatch,
            ScriptedReply::Error(ProtocolError::Timeout),
        );
    }
    let error = write_vsfr_batch(&mut device, &[(VirtSfr::DispBrt, 2)])
        .await
        .expect_err("exhausted");
    assert!(matches!(error, Error::Protocol(ProtocolError::Timeout)));
}
