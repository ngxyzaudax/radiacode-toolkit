use radiacode_protocol::{BytesBuffer, framed_request_header};
use rusb::{Context, DeviceHandle};
use tracing::{debug, warn};

use crate::constants::{EP_OUT, TIMEOUT};
use crate::usb_error::UsbError;

use super::read::{drain_handle, read_matching_response};

pub fn execute_sync(
    handle: &mut DeviceHandle<Context>,
    request: &[u8],
) -> std::result::Result<BytesBuffer, UsbError> {
    let expected =
        framed_request_header(request).map_err(|error| UsbError::Transport(error.to_string()))?;
    debug!(request_len = request.len(), "usb execute request");
    drain_handle(handle)?;
    write_request(handle, request)?;
    read_matching_response(handle, expected)
}

fn write_request(
    handle: &mut DeviceHandle<Context>,
    request: &[u8],
) -> std::result::Result<(), UsbError> {
    let written = handle.write_bulk(EP_OUT, request, TIMEOUT)?;
    if written != request.len() {
        warn!(written, expected = request.len(), "partial usb write");
    }
    Ok(())
}
