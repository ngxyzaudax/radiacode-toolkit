use std::collections::VecDeque;

use radiacode_protocol::{Command, build_request, framed_request_header};

use crate::transport::{drain_with, read_matching_with, read_response_with};
use crate::usb_error::UsbError;

type ReadOutcome = rusb::Result<Vec<u8>>;

fn scripted_reader(script: Vec<ReadOutcome>) -> impl FnMut(&mut [u8]) -> rusb::Result<usize> {
    let mut queue: VecDeque<ReadOutcome> = script.into();
    move |buffer: &mut [u8]| match queue.pop_front() {
        Some(Ok(bytes)) => {
            buffer[..bytes.len()].copy_from_slice(&bytes);
            Ok(bytes.len())
        }
        Some(Err(error)) => Err(error),
        None => Err(rusb::Error::Timeout),
    }
}

fn frame(payload: &[u8]) -> Vec<u8> {
    let mut frame = (payload.len() as i32).to_le_bytes().to_vec();
    frame.extend_from_slice(payload);
    frame
}

fn header_for(command: Command, sequence: u8) -> [u8; 4] {
    framed_request_header(&build_request(command, sequence, &[])).expect("framed request")
}

fn payload_with(header: [u8; 4], body: &[u8]) -> Vec<u8> {
    let mut payload = header.to_vec();
    payload.extend_from_slice(body);
    payload
}

#[test]
fn response_spanning_two_reads_is_reassembled() {
    let full = frame(&[1, 2, 3, 4, 5, 6, 7, 8]);
    let (head, tail) = full.split_at(6);
    let mut reader = scripted_reader(vec![Ok(head.to_vec()), Ok(tail.to_vec())]);
    let response = read_response_with(&mut reader).expect("response");
    assert_eq!(response.data(), &[1, 2, 3, 4, 5, 6, 7, 8]);
}

#[test]
fn repeated_empty_reads_fail_with_empty_read() {
    let mut reader = scripted_reader(vec![Ok(Vec::new()), Ok(Vec::new()), Ok(Vec::new())]);
    assert!(matches!(
        read_response_with(&mut reader),
        Err(UsbError::EmptyRead)
    ));
}

#[test]
fn timeout_before_any_data_is_empty_read() {
    let mut reader = scripted_reader(vec![Err(rusb::Error::Timeout)]);
    assert!(matches!(
        read_response_with(&mut reader),
        Err(UsbError::EmptyRead)
    ));
}

#[test]
fn timeout_after_partial_frame_surfaces_usb_error() {
    let mut reader = scripted_reader(vec![Ok(frame(&[1, 2, 3])[..5].to_vec())]);
    assert!(matches!(
        read_response_with(&mut reader),
        Err(UsbError::Usb(rusb::Error::Timeout))
    ));
}

#[test]
fn unrelated_frames_are_skipped_until_match() {
    let expected = header_for(Command::GetVersion, 0x81);
    let stale = frame(&payload_with(header_for(Command::GetSerial, 0x82), &[9]));
    let wanted = frame(&payload_with(expected, &[7, 7]));
    let mut reader = scripted_reader(vec![Ok(stale), Ok(wanted)]);
    let response = read_matching_with(&mut reader, expected).expect("matching response");
    assert_eq!(response.size(), 6);
}

#[test]
fn too_many_unrelated_frames_report_mismatch() {
    let expected = header_for(Command::GetVersion, 0x81);
    let stale = frame(&payload_with(header_for(Command::GetSerial, 0x82), &[9]));
    let mut reader = scripted_reader(vec![
        Ok(stale.clone()),
        Ok(stale.clone()),
        Ok(stale.clone()),
        Ok(stale),
    ]);
    assert!(matches!(
        read_matching_with(&mut reader, expected),
        Err(UsbError::ResponseMismatch)
    ));
}

#[test]
fn drain_consumes_until_timeout_or_empty() {
    let mut reader = scripted_reader(vec![Ok(vec![1]), Ok(vec![2]), Err(rusb::Error::Timeout)]);
    assert!(drain_with(&mut reader).is_ok());
    let mut empty = scripted_reader(vec![Ok(Vec::new())]);
    assert!(drain_with(&mut empty).is_ok());
    let mut failing = scripted_reader(vec![Err(rusb::Error::NoDevice)]);
    assert!(matches!(
        drain_with(&mut failing),
        Err(UsbError::Usb(rusb::Error::NoDevice))
    ));
}
