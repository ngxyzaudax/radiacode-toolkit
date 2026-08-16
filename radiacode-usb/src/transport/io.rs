use std::sync::{Arc, Mutex};
use std::time::Duration;

use radiacode_protocol::{BytesBuffer, ResponseAssembler};
use rusb::{Context, DeviceHandle, UsbContext};

use crate::constants::{DRAIN, EMPTY_READ_RETRIES, EP_IN, INTERFACE, PID, READ_BUF, TIMEOUT, VID};
use crate::usb_error::UsbError;

pub type LockedHandle = Arc<Mutex<DeviceHandle<Context>>>;

pub fn open_handle(
    context: &Context,
    serial: Option<&str>,
) -> std::result::Result<DeviceHandle<Context>, UsbError> {
    let candidates: Vec<_> = context
        .devices()?
        .iter()
        .filter(|device| {
            device.device_descriptor().ok().is_some_and(|descriptor| {
                descriptor.vendor_id() == VID && descriptor.product_id() == PID
            })
        })
        .collect();
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
    devices: &'a [rusb::Device<Context>],
    serial: Option<&str>,
) -> std::result::Result<&'a rusb::Device<Context>, UsbError> {
    let Some(target) = serial else {
        return Ok(&devices[0]);
    };
    if let Some(address) = target.strip_prefix("usb-") {
        if let Some(device) = devices
            .iter()
            .find(|device| device.address().to_string() == address)
        {
            return Ok(device);
        }
    } else if let Some(device) = devices.iter().find(|device| {
        device
            .device_descriptor()
            .ok()
            .and_then(|descriptor| {
                read_string(None, device, descriptor.serial_number_string_index())
            })
            .is_some_and(|value| value == target)
    }) {
        return Ok(device);
    }
    if devices.len() == 1 {
        return Ok(&devices[0]);
    }
    Err(UsbError::DeviceNotFound)
}

pub fn lock_handle(
    handle: &LockedHandle,
) -> std::result::Result<std::sync::MutexGuard<'_, DeviceHandle<Context>>, UsbError> {
    handle
        .lock()
        .map_err(|_| UsbError::Transport("usb handle lock poisoned".into()))
}

pub fn read_response_sync(
    handle: &mut DeviceHandle<Context>,
) -> std::result::Result<BytesBuffer, UsbError> {
    let mut buffer = vec![0u8; READ_BUF];
    let mut assembler = ResponseAssembler::default();
    let mut empty_reads = 0usize;
    let mut bytes_received = 0usize;
    loop {
        match handle.read_bulk(EP_IN, &mut buffer, TIMEOUT) {
            Ok(0) => {
                empty_reads += 1;
                if empty_reads >= EMPTY_READ_RETRIES {
                    return Err(UsbError::EmptyRead);
                }
            }
            Ok(read) => {
                empty_reads = 0;
                bytes_received += read;
                if let Some(payload) = assembler
                    .push(&buffer[..read])
                    .map_err(response_assembler_error)?
                {
                    return Ok(BytesBuffer::new(payload));
                }
            }
            Err(rusb::Error::Timeout) if bytes_received < 4 => return Err(UsbError::EmptyRead),
            Err(error) => return Err(error.into()),
        }
    }
}

fn response_assembler_error(error: radiacode_protocol::Error) -> UsbError {
    UsbError::Transport(error.to_string())
}

pub fn drain_handle(handle: &mut DeviceHandle<Context>) -> std::result::Result<(), UsbError> {
    let mut buffer = vec![0u8; READ_BUF];
    loop {
        match handle.read_bulk(EP_IN, &mut buffer, DRAIN) {
            Ok(0) | Err(rusb::Error::Timeout) => return Ok(()),
            Err(error) => return Err(error.into()),
            Ok(_) => {}
        }
    }
}

pub fn read_string<T: UsbContext>(
    handle: Option<&DeviceHandle<T>>,
    device: &rusb::Device<T>,
    index: Option<u8>,
) -> Option<String> {
    let index = index?;
    if index == 0 {
        return None;
    }
    let read = |opened: &DeviceHandle<T>| {
        let languages = opened.read_languages(Duration::from_millis(100)).ok()?;
        let language = *languages.first()?;
        opened
            .read_string_descriptor(language, index, Duration::from_millis(100))
            .ok()
    };
    if let Some(handle) = handle {
        return read(handle);
    }
    device.open().ok().and_then(|opened| read(&opened))
}
