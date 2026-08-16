use radiacode_core::{DeviceEndpoint, DiscoveredDevice, model_from_serial};
use rusb::{Context, UsbContext};
use tracing::{debug, info};

use crate::constants::{PID, VID};
use crate::usb_error::UsbError;

use super::io::{open_handle, read_string};

pub fn scan_usb_devices() -> std::result::Result<Vec<DiscoveredDevice>, UsbError> {
    info!("starting usb scan");
    let context = Context::new()?;
    collect_devices(&context)
}

pub fn usb_permission_denied() -> bool {
    let Ok(context) = Context::new() else {
        return false;
    };
    matches!(open_handle(&context, None), Err(UsbError::PermissionDenied))
}

fn collect_devices(context: &Context) -> std::result::Result<Vec<DiscoveredDevice>, UsbError> {
    let mut found = Vec::new();
    for device in context.devices()?.iter() {
        let descriptor = match device.device_descriptor() {
            Ok(value) => value,
            Err(_) => continue,
        };
        if descriptor.vendor_id() != VID || descriptor.product_id() != PID {
            continue;
        }
        let handle = device.open().ok();
        let serial = read_string(
            handle.as_ref(),
            &device,
            descriptor.serial_number_string_index(),
        );
        let product = read_string(handle.as_ref(), &device, descriptor.product_string_index());
        let serial = serial.filter(|value| !value.is_empty());
        let product = product.filter(|value| !value.is_empty());
        let model = serial.as_deref().map(model_from_serial);
        let label = model
            .clone()
            .or_else(|| serial.clone())
            .or(product.clone())
            .unwrap_or_else(|| "RadiaCode".into());
        let endpoint_serial = serial
            .clone()
            .unwrap_or_else(|| format!("usb-{}", device.address()));
        debug!(
            ?serial,
            ?product,
            address = device.address(),
            "matched usb radiacode"
        );
        found.push(DiscoveredDevice {
            endpoint: DeviceEndpoint::Usb {
                serial: endpoint_serial,
            },
            label,
            serial,
            model,
            rssi: None,
        });
    }
    found.sort_by(|left, right| {
        left.endpoint
            .address_label()
            .cmp(right.endpoint.address_label())
    });
    found.dedup_by(|left, right| left.endpoint == right.endpoint);
    info!(count = found.len(), "usb scan complete");
    Ok(found)
}
