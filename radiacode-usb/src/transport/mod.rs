use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use radiacode_protocol::{BytesBuffer, Transport};
use rusb::Context;
use tracing::info;

use crate::usb_error::{UsbError, map_usb_protocol_error};

mod discovery;
mod exchange;
mod identity;
mod open;
mod read;
mod session;

pub use discovery::scan_usb_devices;
pub(crate) use discovery::usb_permission_denied;
pub(crate) use open::is_radiacode_device;
pub use session::{connect, reconnect_session};

#[cfg(test)]
pub(crate) use identity::{UsbIdentity, discovered_device, select_device_index, sort_and_dedup};
#[cfg(test)]
pub(crate) use read::{drain_with, read_matching_with, read_response_with};

use exchange::execute_sync;
use open::{LockedHandle, lock_handle, open_handle};
use read::drain_handle;

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
}

#[async_trait(?Send)]
impl Transport for UsbTransport {
    async fn execute(&mut self, request: &[u8]) -> radiacode_protocol::Result<BytesBuffer> {
        let handle = Arc::clone(&self.handle);
        let request = request.to_vec();
        tokio::task::spawn_blocking(move || {
            let mut handle = lock_handle(&handle).map_err(map_usb_protocol_error)?;
            execute_sync(&mut handle, &request).map_err(map_usb_protocol_error)
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
