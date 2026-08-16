use radiacode_core::Error;
use radiacode_protocol::Error as ProtocolError;

use crate::usb_error::{UsbError, map_usb_error};

#[test]
fn map_usb_error_permission_denied() {
    let mapped = map_usb_error(UsbError::PermissionDenied);
    assert!(matches!(mapped, Error::UsbPermissionDenied));
}

#[test]
fn map_usb_error_device_not_found() {
    let mapped = map_usb_error(UsbError::DeviceNotFound);
    assert!(matches!(mapped, Error::DeviceNotFound));
}

#[test]
fn map_usb_error_empty_read_is_connection_closed() {
    let mapped = map_usb_error(UsbError::EmptyRead);
    assert!(matches!(
        mapped,
        Error::Protocol(ProtocolError::ConnectionClosed)
    ));
}
