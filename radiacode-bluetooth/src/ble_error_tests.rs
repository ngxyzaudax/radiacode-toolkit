use radiacode_core::Error;
use radiacode_protocol::Error as ProtocolError;

use crate::ble_error::{BleError, map_ble_error};

#[test]
fn map_ble_error_device_not_found() {
    let mapped = map_ble_error(BleError::DeviceNotFound);
    assert!(matches!(mapped, Error::DeviceNotFound));
}

#[test]
fn map_ble_error_adapter_not_found_is_transport_unavailable() {
    let mapped = map_ble_error(BleError::AdapterNotFound);
    assert!(matches!(
        mapped,
        Error::Protocol(ProtocolError::TransportUnavailable(_))
    ));
}
