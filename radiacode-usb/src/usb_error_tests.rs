use radiacode_core::Error;
use radiacode_protocol::Error as ProtocolError;

use crate::usb_error::{UsbError, is_connection_lost, map_usb_error, map_usb_protocol_error};

#[test]
fn map_usb_error_permission_denied() {
    let mapped = map_usb_error(UsbError::PermissionDenied);
    assert!(matches!(mapped, Error::UsbPermissionDenied));
    assert!(matches!(
        map_usb_error(UsbError::Usb(rusb::Error::Access)),
        Error::UsbPermissionDenied
    ));
}

#[test]
fn map_usb_error_device_not_found() {
    let mapped = map_usb_error(UsbError::DeviceNotFound);
    assert!(matches!(mapped, Error::DeviceNotFound));
}

#[test]
fn map_usb_error_link_failures_are_connection_closed() {
    for error in [
        UsbError::EmptyRead,
        UsbError::ResponseMismatch,
        UsbError::Usb(rusb::Error::NoDevice),
    ] {
        assert!(matches!(
            map_usb_error(error),
            Error::Protocol(ProtocolError::ConnectionClosed)
        ));
    }
}

#[test]
fn map_usb_error_timeout_and_transport() {
    assert!(matches!(
        map_usb_error(UsbError::Usb(rusb::Error::Timeout)),
        Error::Protocol(ProtocolError::Timeout)
    ));
    assert!(matches!(
        map_usb_error(UsbError::Transport("lock".into())),
        Error::Protocol(ProtocolError::TransportUnavailable(message)) if message == "lock"
    ));
    assert!(matches!(
        map_usb_error(UsbError::Usb(rusb::Error::Pipe)),
        Error::Protocol(ProtocolError::TransportUnavailable(_))
    ));
}

#[test]
fn map_usb_protocol_error_covers_all_variants() {
    assert!(matches!(
        map_usb_protocol_error(UsbError::EmptyRead),
        ProtocolError::ConnectionClosed
    ));
    assert!(matches!(
        map_usb_protocol_error(UsbError::Usb(rusb::Error::Timeout)),
        ProtocolError::Timeout
    ));
    for error in [
        UsbError::DeviceNotFound,
        UsbError::PermissionDenied,
        UsbError::Usb(rusb::Error::Access),
        UsbError::Usb(rusb::Error::Io),
        UsbError::Transport("x".into()),
    ] {
        assert!(matches!(
            map_usb_protocol_error(error),
            ProtocolError::TransportUnavailable(_)
        ));
    }
}

#[test]
fn connection_lost_includes_permission_denied() {
    assert!(is_connection_lost(&Error::UsbPermissionDenied));
    assert!(is_connection_lost(&Error::Protocol(
        ProtocolError::ConnectionClosed
    )));
    assert!(!is_connection_lost(&Error::DeviceNotFound));
}
