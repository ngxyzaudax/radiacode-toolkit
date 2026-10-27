//! Linux BlueZ management-socket RSSI sampling.
//!
//! `radiacode-lib` reports the RSSI cached by btleplug. On Linux the desktop
//! app also asks the BlueZ management socket for the connected link's RSSI.
//! That socket requires `CAP_NET_ADMIN`. Membership in the `bluetooth` group is
//! enough for `bluetoothctl` and is not enough for this socket. A refusal is
//! [`radiacode_lib::RssiStatus::PermissionDenied`], displayed as
//! `RSSI unavailable: permission denied`, unless a cached measurement exists.

mod sampling;
mod status;

#[cfg(target_os = "linux")]
mod mgmt;

pub use sampling::SamplingBluetoothTransport;
pub use status::{MgmtRssi, combine_rssi};

/// Queries the BlueZ management socket, or reports [`MgmtRssi::Unavailable`] off Linux.
pub async fn read_mgmt_rssi(mac: &str) -> MgmtRssi {
    #[cfg(target_os = "linux")]
    {
        mgmt::read_connected_rssi(mac).await
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = mac;
        MgmtRssi::Unavailable
    }
}
