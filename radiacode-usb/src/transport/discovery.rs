use radiacode_core::DiscoveredDevice;
use rusb::{Context, Device};
use tracing::{debug, info};

use crate::usb_error::UsbError;

use super::identity::{UsbIdentity, discovered_device, sort_and_dedup};
use super::open::{open_handle, radiacode_devices, read_string};

pub fn scan_usb_devices() -> std::result::Result<Vec<DiscoveredDevice>, UsbError> {
    info!("starting usb scan");
    let context = Context::new()?;
    let found: Vec<DiscoveredDevice> = radiacode_devices(&context)?
        .iter()
        .map(|device| discovered_device(identity_of(device)))
        .collect();
    let found = sort_and_dedup(found);
    info!(count = found.len(), "usb scan complete");
    Ok(found)
}

pub fn usb_permission_denied() -> bool {
    let Ok(context) = Context::new() else {
        return false;
    };
    matches!(open_handle(&context, None), Err(UsbError::PermissionDenied))
}

fn identity_of(device: &Device<Context>) -> UsbIdentity {
    let descriptor = device.device_descriptor().ok();
    let handle = device.open().ok();
    let serial = descriptor.as_ref().and_then(|descriptor| {
        read_string(
            handle.as_ref(),
            device,
            descriptor.serial_number_string_index(),
        )
    });
    let product = descriptor.as_ref().and_then(|descriptor| {
        read_string(handle.as_ref(), device, descriptor.product_string_index())
    });
    debug!(
        ?serial,
        ?product,
        address = device.address(),
        "matched usb radiacode"
    );
    UsbIdentity {
        address: device.address(),
        serial,
        product,
    }
}
