use std::future::Future;

use radiacode_lib::usb::UsbAccessStatus;
use radiacode_lib::{DiscoveredDevice, Error};

/// Probes USB access from the UI thread, which does not own the worker runtime.
pub fn access_status() -> UsbAccessStatus {
    match block_on(radiacode_lib::usb::access_status()) {
        Ok(Ok(status)) => status,
        Ok(Err(_)) | Err(_) => UsbAccessStatus::Granted,
    }
}

/// Enumerates USB devices from the UI thread.
pub fn scan_usb_devices() -> Result<Vec<DiscoveredDevice>, Error> {
    match block_on(radiacode_lib::usb::scan_usb_devices()) {
        Ok(result) => result,
        Err(message) => Err(Error::TransportUnavailable(message)),
    }
}

fn block_on<T>(future: impl Future<Output = T>) -> Result<T, String> {
    if tokio::runtime::Handle::try_current().is_ok() {
        return Err("usb probe was called from inside a running tokio runtime".into());
    }
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| error.to_string())?;
    Ok(runtime.block_on(future))
}
