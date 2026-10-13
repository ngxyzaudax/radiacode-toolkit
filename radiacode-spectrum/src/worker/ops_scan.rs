use std::time::Duration;

use crossbeam_channel::Sender;
use radiacode_bluetooth::{BleError, scan_radiacode_devices};
use radiacode_core::{DiscoveredDevice, merge_discovered};
use radiacode_usb::scan_usb_devices;
use tracing::{info, warn};

use super::messages::WorkerEvent;

const BLUETOOTH_SCAN_DURATION: Duration = Duration::from_secs(5);

pub async fn handle_scan(events: &Sender<WorkerEvent>) {
    info!("scanning for radiacode devices over usb and bluetooth");
    let (usb_devices, ble_devices) = tokio::join!(scan_usb_async(), scan_bluetooth_async());
    let devices = merge_scan_results(
        devices_or_empty(usb_devices, "usb"),
        devices_or_empty(ble_devices, "bluetooth"),
    );
    info!(count = devices.len(), "scan finished");
    let _ = events.send(WorkerEvent::ScanFinished(devices));
}

fn devices_or_empty(
    outcome: Result<Vec<DiscoveredDevice>, String>,
    transport: &str,
) -> Vec<DiscoveredDevice> {
    outcome.unwrap_or_else(|error| {
        warn!(%error, transport, "device scan failed");
        Vec::new()
    })
}

fn merge_scan_results(
    usb_devices: Vec<DiscoveredDevice>,
    ble_devices: Vec<DiscoveredDevice>,
) -> Vec<DiscoveredDevice> {
    if usb_devices.is_empty() && ble_devices.is_empty() {
        return Vec::new();
    }
    merge_discovered(usb_devices, ble_devices)
}

async fn scan_usb_async() -> Result<Vec<DiscoveredDevice>, String> {
    tokio::task::spawn_blocking(scan_usb_devices)
        .await
        .map_err(|error| error.to_string())?
        .map_err(|error| error.to_string())
}

async fn scan_bluetooth_async() -> Result<Vec<DiscoveredDevice>, String> {
    scan_radiacode_devices(BLUETOOTH_SCAN_DURATION)
        .await
        .map_err(|error: BleError| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::{devices_or_empty, merge_scan_results};
    use radiacode_core::{DeviceEndpoint, DiscoveredDevice};

    fn usb_device(serial: &str) -> DiscoveredDevice {
        DiscoveredDevice {
            endpoint: DeviceEndpoint::Usb {
                serial: serial.to_string(),
            },
            label: format!("RadiaCode {serial}"),
            serial: Some(serial.to_string()),
            model: None,
            rssi: None,
        }
    }

    #[test]
    fn failed_scan_yields_empty_list() {
        assert!(devices_or_empty(Err("boom".to_string()), "usb").is_empty());
        assert_eq!(devices_or_empty(Ok(vec![usb_device("A")]), "usb").len(), 1);
    }

    #[test]
    fn merge_of_empty_results_is_empty() {
        assert!(merge_scan_results(Vec::new(), Vec::new()).is_empty());
        assert_eq!(
            merge_scan_results(vec![usb_device("A")], Vec::new()).len(),
            1
        );
    }
}
