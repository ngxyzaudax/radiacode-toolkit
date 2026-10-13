mod adapter;
mod address;
mod advertisement;
mod ble_error;
mod drain;
mod execute;
mod link;
mod rssi;
mod scan;
mod scan_session;
mod transport;
mod transport_connect;
mod uuids;

pub use ble_error::{BleError, is_connection_lost, map_ble_error};
pub use radiacode_core::{
    AlarmLimits, AlarmLimitsUpdate, DeviceEndpoint, DeviceMetadata, DeviceStatus, DiscoveredDevice,
    Error, LiveRates, RadiaCode, Result, SessionRestore, Spectrum, Transport, TransportKind,
    merge_discovered,
};
pub use rssi::read_connected_rssi_dbm;
pub use scan::scan_radiacode_devices;
pub use transport::{BluetoothTransport, connect, reconnect_session};

#[cfg(test)]
mod address_tests;
#[cfg(test)]
mod advertisement_tests;
#[cfg(test)]
mod ble_error_tests;
#[cfg(test)]
mod drain_tests;
#[cfg(test)]
mod execute_tests;
#[cfg(test)]
mod test_streams;
