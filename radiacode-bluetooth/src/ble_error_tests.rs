use radiacode_core::Error;
use radiacode_protocol::Error as ProtocolError;

use crate::ble_error::{
    BleError, is_bluetooth_connection_lost, is_connection_lost, map_ble_error,
    map_ble_protocol_error,
};

fn transport_message(error: ProtocolError) -> String {
    match error {
        ProtocolError::TransportUnavailable(message) => message,
        other => panic!("expected transport unavailable, got {other:?}"),
    }
}

fn core_transport_message(error: Error) -> String {
    match error {
        Error::Protocol(protocol) => transport_message(protocol),
        other => panic!("expected protocol error, got {other:?}"),
    }
}

#[test]
fn map_ble_error_device_not_found() {
    let mapped = map_ble_error(BleError::DeviceNotFound);
    assert!(matches!(mapped, Error::DeviceNotFound));
}

#[test]
fn map_ble_error_adapter_not_found_is_transport_unavailable() {
    let mapped = map_ble_error(BleError::AdapterNotFound);
    assert_eq!(
        core_transport_message(mapped),
        "bluetooth adapter not found"
    );
}

#[test]
fn map_ble_error_characteristic_and_address_variants() {
    assert_eq!(
        core_transport_message(map_ble_error(BleError::CharacteristicMissing)),
        "required BLE characteristic missing"
    );
    assert_eq!(
        core_transport_message(map_ble_error(BleError::InvalidAddress("zz".into()))),
        "invalid bluetooth address: zz"
    );
}

#[test]
fn map_ble_error_bluetooth_link_loss_becomes_connection_closed() {
    let mapped = map_ble_error(BleError::Bluetooth(btleplug::Error::NotConnected));
    assert!(mapped.is_connection_lost());
}

#[test]
fn map_ble_error_other_bluetooth_errors_keep_message() {
    let mapped = map_ble_error(BleError::Bluetooth(btleplug::Error::PermissionDenied));
    assert_eq!(core_transport_message(mapped), "Permission denied");
}

#[test]
fn map_ble_protocol_error_covers_every_variant() {
    assert!(matches!(
        map_ble_protocol_error(BleError::Bluetooth(btleplug::Error::NotConnected)),
        ProtocolError::ConnectionClosed
    ));
    assert_eq!(
        transport_message(map_ble_protocol_error(BleError::DeviceNotFound)),
        "device not found"
    );
    assert_eq!(
        transport_message(map_ble_protocol_error(BleError::AdapterNotFound)),
        "bluetooth adapter not found"
    );
    assert_eq!(
        transport_message(map_ble_protocol_error(BleError::CharacteristicMissing)),
        "required BLE characteristic missing"
    );
    assert_eq!(
        transport_message(map_ble_protocol_error(BleError::InvalidAddress(
            "bad".into()
        ))),
        "invalid bluetooth address: bad"
    );
    assert_eq!(
        transport_message(map_ble_protocol_error(BleError::Bluetooth(
            btleplug::Error::RuntimeError("busy".into())
        ))),
        "Runtime Error: busy"
    );
}

#[test]
fn bluetooth_connection_lost_detects_link_messages() {
    assert!(is_bluetooth_connection_lost(&btleplug::Error::NotConnected));
    assert!(is_bluetooth_connection_lost(
        &btleplug::Error::DeviceNotFound
    ));
    assert!(is_bluetooth_connection_lost(
        &btleplug::Error::RuntimeError("Link has been lost".into())
    ));
    assert!(is_bluetooth_connection_lost(
        &btleplug::Error::RuntimeError("Broken pipe".into())
    ));
    assert!(is_bluetooth_connection_lost(
        &btleplug::Error::RuntimeError("Connection reset by peer".into())
    ));
    assert!(!is_bluetooth_connection_lost(
        &btleplug::Error::PermissionDenied
    ));
}

#[test]
fn connection_lost_inspects_transport_messages() {
    assert!(is_connection_lost(&Error::connection_closed()));
    assert!(is_connection_lost(&Error::from(
        ProtocolError::TransportUnavailable("Device Disconnected".into())
    )));
    assert!(is_connection_lost(&Error::from(
        ProtocolError::TransportUnavailable("link has been lost".into())
    )));
    assert!(!is_connection_lost(&Error::from(
        ProtocolError::TransportUnavailable("busy".into())
    )));
    assert!(!is_connection_lost(&Error::timeout()));
}
