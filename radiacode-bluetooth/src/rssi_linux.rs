use tracing::debug;

use super::rssi_mgmt;
use crate::address::{bluez_device_path, parse_dbus_variant_int16};

pub async fn read_connected_rssi_dbm(mac: &str) -> Option<i16> {
    if let Some(rssi) = rssi_mgmt::read_connected_rssi_dbm(mac).await {
        return Some(rssi);
    }
    read_dbus_rssi_dbm(mac)
}

pub fn read_dbus_rssi_dbm(mac: &str) -> Option<i16> {
    let path = bluez_device_path(mac);
    let output = std::process::Command::new("dbus-send")
        .args([
            "--system",
            "--print-reply",
            "--dest=org.bluez",
            &path,
            "org.freedesktop.DBus.Properties.Get",
            "string:org.bluez.Device1",
            "string:RSSI",
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let rssi = parse_dbus_variant_int16(&output.stdout)?;
    debug!(%mac, rssi, "rssi from bluez property fallback");
    Some(rssi)
}
