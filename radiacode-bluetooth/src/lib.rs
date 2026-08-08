mod adapter;
mod ble_error;
mod device_model;
mod execute;
mod link;
mod rssi;
mod scan;
mod transport;
mod uuids;

pub use ble_error::{is_connection_lost, map_ble_error, BleError};
pub use radiacode_core::{
    merge_discovered, AlarmLimits, AlarmLimitsUpdate, DeviceEndpoint, DeviceMetadata,
    DeviceStatus, DiscoveredDevice, Error, LiveRates, RadiaCode, Result, SessionRestore,
    Spectrum, Transport, TransportKind,
};
pub use scan::scan_radiacode_devices;
pub use transport::{connect, reconnect_session, BluetoothTransport};
pub use rssi::read_connected_rssi_dbm;
