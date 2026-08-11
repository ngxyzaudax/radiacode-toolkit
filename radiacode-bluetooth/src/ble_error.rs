use thiserror::Error;

#[derive(Debug, Error)]
pub enum BleError {
    #[error("bluetooth adapter not found")]
    AdapterNotFound,
    #[error("device not found")]
    DeviceNotFound,
    #[error("required BLE characteristic missing")]
    CharacteristicMissing,
    #[error("invalid bluetooth address: {0}")]
    InvalidAddress(String),
    #[error(transparent)]
    Bluetooth(#[from] btleplug::Error),
}

pub fn map_ble_error(error: BleError) -> radiacode_core::Error {
    match error {
        BleError::AdapterNotFound => radiacode_core::Error::from(
            radiacode_protocol::Error::TransportUnavailable("bluetooth adapter not found".into()),
        ),
        BleError::DeviceNotFound => radiacode_core::Error::DeviceNotFound,
        BleError::CharacteristicMissing => radiacode_core::Error::from(
            radiacode_protocol::Error::TransportUnavailable("required BLE characteristic missing".into()),
        ),
        BleError::InvalidAddress(value) => radiacode_core::Error::from(
            radiacode_protocol::Error::TransportUnavailable(format!("invalid bluetooth address: {value}")),
        ),
        BleError::Bluetooth(error) if is_bluetooth_connection_lost(&error) => {
            radiacode_core::Error::from(radiacode_protocol::Error::ConnectionClosed)
        }
        BleError::Bluetooth(error) => {
            radiacode_core::Error::from(radiacode_protocol::Error::TransportUnavailable(error.to_string()))
        }
    }
}

pub fn map_ble_protocol_error(error: BleError) -> radiacode_protocol::Error {
    match error {
        BleError::Bluetooth(error) if is_bluetooth_connection_lost(&error) => {
            radiacode_protocol::Error::ConnectionClosed
        }
        BleError::DeviceNotFound => {
            radiacode_protocol::Error::TransportUnavailable("device not found".into())
        }
        BleError::AdapterNotFound => {
            radiacode_protocol::Error::TransportUnavailable("bluetooth adapter not found".into())
        }
        BleError::CharacteristicMissing => {
            radiacode_protocol::Error::TransportUnavailable("required BLE characteristic missing".into())
        }
        BleError::InvalidAddress(value) => {
            radiacode_protocol::Error::TransportUnavailable(format!("invalid bluetooth address: {value}"))
        }
        BleError::Bluetooth(error) => radiacode_protocol::Error::TransportUnavailable(error.to_string()),
    }
}

pub fn is_bluetooth_connection_lost(error: &btleplug::Error) -> bool {
    let message = error.to_string().to_ascii_lowercase();
    message.contains("not connected")
        || message.contains("disconnected")
        || message.contains("device not found")
        || message.contains("link has been lost")
        || message.contains("broken pipe")
        || message.contains("connection reset")
}

pub fn is_connection_lost(error: &radiacode_core::Error) -> bool {
    error.is_connection_lost()
        || matches!(
            error,
            radiacode_core::Error::Protocol(radiacode_protocol::Error::TransportUnavailable(message))
                if message.to_ascii_lowercase().contains("not connected")
                    || message.to_ascii_lowercase().contains("disconnected")
                    || message.to_ascii_lowercase().contains("link has been lost")
        )
}
