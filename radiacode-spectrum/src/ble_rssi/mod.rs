//! Linux BlueZ management-socket RSSI sampling.
//!
//! `radiacode-lib` reports the RSSI cached by btleplug. The desktop app also
//! asks BlueZ for the connected link's RSSI, which tracks a live session more
//! closely than the last advertisement.

mod sampling;

#[cfg(target_os = "linux")]
mod mgmt;

pub use sampling::SamplingBluetoothTransport;

pub async fn read_mgmt_rssi_dbm(mac: &str) -> Option<i16> {
    #[cfg(target_os = "linux")]
    {
        mgmt::read_connected_rssi_dbm(mac).await
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = mac;
        None
    }
}
