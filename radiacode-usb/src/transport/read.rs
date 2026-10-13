use radiacode_protocol::{BytesBuffer, ResponseAssembler, response_matches_request};
use rusb::{Context, DeviceHandle};
use tracing::warn;

use crate::constants::{DRAIN, EMPTY_READ_RETRIES, EP_IN, READ_BUF, TIMEOUT};
use crate::usb_error::UsbError;

const MAX_UNRELATED_FRAMES: usize = 4;
const MIN_FRAME_PREFIX: usize = 4;

pub type BulkReader<'a> = dyn FnMut(&mut [u8]) -> rusb::Result<usize> + 'a;

pub fn read_matching_response(
    handle: &mut DeviceHandle<Context>,
    expected: [u8; 4],
) -> std::result::Result<BytesBuffer, UsbError> {
    read_matching_with(
        &mut |buffer| handle.read_bulk(EP_IN, buffer, TIMEOUT),
        expected,
    )
}

pub fn read_matching_with(
    reader: &mut BulkReader<'_>,
    expected: [u8; 4],
) -> std::result::Result<BytesBuffer, UsbError> {
    for discarded in 0..MAX_UNRELATED_FRAMES {
        let payload = read_response_with(reader)?;
        if response_matches_request(payload.data(), expected) {
            return Ok(payload);
        }
        warn!(
            response_len = payload.size(),
            discarded = discarded + 1,
            "discarding unrelated usb frame"
        );
    }
    Err(UsbError::ResponseMismatch)
}

pub fn read_response_with(
    reader: &mut BulkReader<'_>,
) -> std::result::Result<BytesBuffer, UsbError> {
    let mut buffer = vec![0u8; READ_BUF];
    let mut assembler = ResponseAssembler::default();
    let mut empty_reads = 0usize;
    let mut bytes_received = 0usize;
    loop {
        match reader(&mut buffer) {
            Ok(0) => {
                empty_reads += 1;
                if empty_reads >= EMPTY_READ_RETRIES {
                    return Err(UsbError::EmptyRead);
                }
            }
            Ok(read) => {
                empty_reads = 0;
                bytes_received += read;
                if let Some(payload) = push_chunk(&mut assembler, &buffer[..read])? {
                    return Ok(payload);
                }
            }
            Err(rusb::Error::Timeout) if bytes_received < MIN_FRAME_PREFIX => {
                return Err(UsbError::EmptyRead);
            }
            Err(error) => return Err(error.into()),
        }
    }
}

fn push_chunk(
    assembler: &mut ResponseAssembler,
    chunk: &[u8],
) -> std::result::Result<Option<BytesBuffer>, UsbError> {
    assembler
        .push(chunk)
        .map(|payload| payload.map(BytesBuffer::new))
        .map_err(|error| UsbError::Transport(error.to_string()))
}

pub fn drain_handle(handle: &mut DeviceHandle<Context>) -> std::result::Result<(), UsbError> {
    drain_with(&mut |buffer| handle.read_bulk(EP_IN, buffer, DRAIN))
}

pub fn drain_with(reader: &mut BulkReader<'_>) -> std::result::Result<(), UsbError> {
    let mut buffer = vec![0u8; READ_BUF];
    loop {
        match reader(&mut buffer) {
            Ok(0) | Err(rusb::Error::Timeout) => return Ok(()),
            Err(error) => return Err(error.into()),
            Ok(_) => {}
        }
    }
}
