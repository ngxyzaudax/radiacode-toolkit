use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use rusb::{Context, Device, DeviceHandle, UsbContext};

use crate::constants::{INTERFACE, PID, VID};
use crate::transport::identity::select_device_index;
use crate::transport::read::drain_handle;
use crate::usb_error::UsbError;

const STRING_READ_TIMEOUT: Duration = Duration::from_millis(100);

pub type LockedHandle = Arc<Mutex<DeviceHandle<Context>>>;

pub fn is_radiacode_device<T: UsbContext>(device: &Device<T>) -> bool {
    device
        .device_descriptor()
        .ok()
        .is_some_and(|descriptor| descriptor.vendor_id() == VID && descriptor.product_id() == PID)
}

pub fn radiacode_devices(context: &Context) -> std::result::Result<Vec<Device<Context>>, UsbError> {
    Ok(context
        .devices()?
        .iter()
        .filter(is_radiacode_device)
        .collect())
}

pub fn open_handle(
    context: &Context,
    serial: Option<&str>,
) -> std::result::Result<DeviceHandle<Context>, UsbError> {
    let candidates = radiacode_devices(context)?;
    if candidates.is_empty() {
        return Err(UsbError::DeviceNotFound);
    }
    let device = pick_device(&candidates, serial)?;
    let mut handle = match device.open() {
        Ok(value) => value,
        Err(rusb::Error::Access) => return Err(UsbError::PermissionDenied),
        Err(error) => return Err(error.into()),
    };
    if handle.kernel_driver_active(INTERFACE).unwrap_or(false) {
        handle.detach_kernel_driver(INTERFACE).ok();
    }
    handle.claim_interface(INTERFACE)?;
    drain_handle(&mut handle)?;
    Ok(handle)
}

fn pick_device<'a>(
    devices: &'a [Device<Context>],
    serial: Option<&str>,
) -> std::result::Result<&'a Device<Context>, UsbError> {
    let Some(target) = serial else {
        return Ok(&devices[0]);
    };
    let addresses: Vec<String> = devices
        .iter()
        .map(|device| device.address().to_string())
        .collect();
    let serials: Vec<Option<String>> = devices.iter().map(device_serial).collect();
    select_device_index(target, &addresses, &serials)
        .map(|index| &devices[index])
        .ok_or(UsbError::DeviceNotFound)
}

fn device_serial(device: &Device<Context>) -> Option<String> {
    let descriptor = device.device_descriptor().ok()?;
    read_string(None, device, descriptor.serial_number_string_index())
}

pub fn lock_handle(
    handle: &LockedHandle,
) -> std::result::Result<MutexGuard<'_, DeviceHandle<Context>>, UsbError> {
    handle
        .lock()
        .map_err(|_| UsbError::Transport("usb handle lock poisoned".into()))
}

pub fn read_string<T: UsbContext>(
    handle: Option<&DeviceHandle<T>>,
    device: &Device<T>,
    index: Option<u8>,
) -> Option<String> {
    let index = index.filter(|index| *index != 0)?;
    match handle {
        Some(opened) => read_string_descriptor(opened, index),
        None => device
            .open()
            .ok()
            .and_then(|opened| read_string_descriptor(&opened, index)),
    }
}

fn read_string_descriptor<T: UsbContext>(opened: &DeviceHandle<T>, index: u8) -> Option<String> {
    let languages = opened.read_languages(STRING_READ_TIMEOUT).ok()?;
    let language = *languages.first()?;
    opened
        .read_string_descriptor(language, index, STRING_READ_TIMEOUT)
        .ok()
}
