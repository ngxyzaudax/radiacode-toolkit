use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use radiacode_core::{Error, RadiaCode, SessionRestore};
use radiacode_protocol::{BytesBuffer, Transport, framed_request_header, response_matches_request};
use rusb::{Context, DeviceHandle};
use tracing::{debug, info};

use crate::constants::{EP_OUT, TIMEOUT};
use crate::usb_error::{UsbError, map_usb_error, map_usb_protocol_error};

mod discovery;
mod io;

pub use discovery::scan_usb_devices;
pub(crate) use discovery::usb_permission_denied;
pub use io::{LockedHandle, drain_handle, lock_handle, open_handle, read_response_sync};

pub struct UsbTransport {
    _context: Context,
    handle: LockedHandle,
}

impl UsbTransport {
    pub fn connect(serial: &str) -> std::result::Result<Self, UsbError> {
        info!(%serial, "usb transport connect");
        let context = Context::new()?;
        let handle = open_handle(&context, Some(serial))?;
        Ok(Self {
            _context: context,
            handle: Arc::new(Mutex::new(handle)),
        })
    }

    pub fn reconnect(serial: &str) -> std::result::Result<Self, UsbError> {
        Self::connect(serial)
    }

    fn execute_sync(
        handle: &mut DeviceHandle<Context>,
        request: &[u8],
    ) -> std::result::Result<BytesBuffer, UsbError> {
        let expected = framed_request_header(request)
            .map_err(|error| UsbError::Transport(error.to_string()))?;
        debug!(request_len = request.len(), "usb execute request");
        drain_handle(handle)?;
        let written = handle.write_bulk(EP_OUT, request, TIMEOUT)?;
        if written != request.len() {
            tracing::warn!(written, expected = request.len(), "partial usb write");
        }
        let payload = read_response_sync(handle)?;
        if !response_matches_request(payload.data(), expected) {
            return Err(UsbError::LengthMismatch);
        }
        Ok(payload)
    }
}

#[async_trait(?Send)]
impl Transport for UsbTransport {
    async fn execute(&mut self, request: &[u8]) -> radiacode_protocol::Result<BytesBuffer> {
        let handle = Arc::clone(&self.handle);
        let request = request.to_vec();
        tokio::task::spawn_blocking(move || {
            let mut handle = lock_handle(&handle).map_err(map_usb_protocol_error)?;
            UsbTransport::execute_sync(&mut handle, &request).map_err(map_usb_protocol_error)
        })
        .await
        .map_err(|error| radiacode_protocol::Error::TransportUnavailable(error.to_string()))?
    }

    async fn drain_link(&mut self) {
        let handle = Arc::clone(&self.handle);
        let _ = tokio::task::spawn_blocking(move || {
            let mut handle = lock_handle(&handle)?;
            drain_handle(&mut handle)
        })
        .await;
    }

    async fn disconnect(self: Box<Self>) -> radiacode_protocol::Result<()> {
        info!("usb transport disconnect");
        Ok(())
    }

    async fn link_rssi_dbm(&self) -> Option<i16> {
        None
    }

    async fn sample_link_rssi_dbm(&self) -> Option<i16> {
        None
    }
}

pub async fn connect(serial: &str) -> radiacode_core::Result<RadiaCode> {
    let transport = tokio::task::spawn_blocking({
        let serial = serial.to_string();
        move || UsbTransport::connect(&serial).map_err(map_usb_error)
    })
    .await
    .map_err(|error| {
        Error::from(radiacode_protocol::Error::TransportUnavailable(
            error.to_string(),
        ))
    })??;
    RadiaCode::open(Box::new(transport), false, None).await
}

pub async fn reconnect_session(
    serial: &str,
    restore: &SessionRestore,
) -> radiacode_core::Result<RadiaCode> {
    info!(%serial, "radiacode usb reconnect with cached session");
    let transport = tokio::task::spawn_blocking({
        let serial = serial.to_string();
        move || UsbTransport::reconnect(&serial).map_err(map_usb_error)
    })
    .await
    .map_err(|error| {
        Error::from(radiacode_protocol::Error::TransportUnavailable(
            error.to_string(),
        ))
    })??;
    RadiaCode::open(Box::new(transport), false, Some(restore)).await
}
