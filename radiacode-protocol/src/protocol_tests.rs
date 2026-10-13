use crate::buffer::BytesBuffer;
use crate::command::Command;
use crate::error::Error;
use crate::protocol::{
    ResponseAssembler, Sequence, build_request, framed_request_header, request_header,
    response_matches_request, strip_echoed_header,
};

#[test]
fn framed_request_header_reads_command_bytes() {
    let request = build_request(Command::GetVersion, 0x81, &[]);
    assert_eq!(
        framed_request_header(&request).unwrap(),
        request_header(Command::GetVersion, 0x81)
    );
}

#[test]
fn framed_request_header_rejects_short_requests() {
    assert!(matches!(
        framed_request_header(&[1, 2, 3]),
        Err(Error::ProtocolMismatch { .. })
    ));
}

#[test]
fn response_match_requires_echoed_header() {
    let header = request_header(Command::GetVersion, 0x81);
    let mut payload = header.to_vec();
    payload.extend_from_slice(&[1, 2, 3]);
    assert!(response_matches_request(&payload, header));
    assert!(!response_matches_request(&[0, 0, 0, 0], header));
    assert!(!response_matches_request(&[0x0a], header));
}

#[test]
fn strip_echoed_header_returns_payload_or_mismatch() {
    let header = request_header(Command::GetStatus, 0x85);
    let mut response = header.to_vec();
    response.extend_from_slice(&[9, 8]);
    let payload = strip_echoed_header(BytesBuffer::new(response), header).unwrap();
    assert_eq!(payload.data(), &[9, 8]);
    let wrong = strip_echoed_header(BytesBuffer::new(vec![0, 0, 0, 0]), header);
    assert!(matches!(wrong, Err(Error::ProtocolMismatch { .. })));
    let short = strip_echoed_header(BytesBuffer::new(vec![0]), header);
    assert!(matches!(
        short,
        Err(Error::BufferUnderrun { need: 4, have: 1 })
    ));
}

#[test]
fn sequence_wraps_within_32() {
    let mut sequence = Sequence::default();
    assert_eq!(sequence.advance(), 0x80);
    for _ in 0..31 {
        let _ = sequence.advance();
    }
    assert_eq!(sequence.advance(), 0x80);
}

#[test]
fn session_start_sequence_is_within_window() {
    let sequence = Sequence::session_start();
    assert!(sequence.start_value() < 32);
}

#[test]
fn build_request_frames_length_and_header() {
    let request = build_request(Command::GetVersion, 0x81, &[7, 7]);
    assert_eq!(&request[..4], &6u32.to_le_bytes());
    assert_eq!(&request[4..8], &request_header(Command::GetVersion, 0x81));
    assert_eq!(&request[8..], &[7, 7]);
}

#[test]
fn assembler_reassembles_chunked_payload() {
    let mut assembler = ResponseAssembler::default();
    let payload = vec![1, 2, 3, 4, 5];
    let mut first = (payload.len() as i32).to_le_bytes().to_vec();
    first.extend_from_slice(&payload[..2]);
    assert!(assembler.push(&first).unwrap().is_none());
    let complete = assembler.push(&payload[2..]).unwrap().unwrap();
    assert_eq!(complete, payload);
}

#[test]
fn assembler_completes_single_chunk_and_resets() {
    let mut assembler = ResponseAssembler::default();
    let mut frame = 2i32.to_le_bytes().to_vec();
    frame.extend_from_slice(&[0xAA, 0xBB]);
    assert_eq!(assembler.push(&frame).unwrap(), Some(vec![0xAA, 0xBB]));
    assert_eq!(assembler.push(&frame).unwrap(), Some(vec![0xAA, 0xBB]));
}

#[test]
fn assembler_rejects_short_and_negative_headers() {
    let mut assembler = ResponseAssembler::default();
    assert!(matches!(
        assembler.push(&[1, 2]),
        Err(Error::BufferUnderrun { need: 4, have: 2 })
    ));
    assert!(matches!(
        assembler.push(&(-1i32).to_le_bytes()),
        Err(Error::ProtocolMismatch { .. })
    ));
}

#[test]
fn bytes_buffer_reads_and_underruns() {
    let mut buffer = BytesBuffer::new(vec![1, 2, 0, 3, 0, 0, 0, 2, b'h', b'i', 0xff]);
    assert_eq!(buffer.take_u8().unwrap(), 1);
    assert_eq!(buffer.take_u16_le().unwrap(), 2);
    assert_eq!(buffer.take_u32_le().unwrap(), 3);
    assert_eq!(buffer.take_length_prefixed_ascii().unwrap(), "hi");
    assert_eq!(buffer.size(), 1);
    assert!(matches!(
        buffer.take_f32_le(),
        Err(Error::BufferUnderrun { need: 4, have: 1 })
    ));
    assert_eq!(buffer.data(), &[0xff]);
    assert!(buffer.skip(1).is_ok());
    assert!(buffer.skip(1).is_err());
    assert!(buffer.into_remaining().is_empty());
}

#[test]
fn bytes_buffer_signed_reads() {
    let mut buffer = BytesBuffer::new((-5i32).to_le_bytes().to_vec());
    assert_eq!(buffer.take_i32_le().unwrap(), -5);
}

#[test]
fn error_classification() {
    assert!(Error::BufferUnderrun { need: 1, have: 0 }.is_transient());
    assert!(
        Error::ProtocolMismatch {
            expected: "a".into(),
            got: "b".into()
        }
        .is_transient()
    );
    assert!(!Error::UnexpectedReturnCode(3).is_transient());
    assert!(!Error::VsfrBatchEmpty.is_transient());
    assert!(!Error::TransportUnavailable("x".into()).is_connection_lost());
    assert_eq!(
        Error::UnexpectedReturnCode(3).to_string(),
        "unexpected device return code 3"
    );
}
