use std::time::Duration;

use btleplug::api::{Central, Peripheral as _, PeripheralProperties, ScanFilter};
use btleplug::platform::Peripheral;
use radiacode_core::DiscoveredDevice;
use tracing::{debug, info};

use crate::adapter::default_adapter;
use crate::advertisement::{
    Advertisement, discovered_device, is_radiacode_advertisement, sort_and_dedup,
};
use crate::ble_error::BleError;

pub async fn scan_radiacode_devices(
    duration: Duration,
) -> std::result::Result<Vec<DiscoveredDevice>, BleError> {
    info!(?duration, "starting ble scan");
    let adapter = default_adapter().await?;
    crate::scan_session::remember_scan_adapter(&adapter).await;
    adapter.start_scan(ScanFilter::default()).await?;
    tokio::time::sleep(duration).await;
    let peripherals = adapter.peripherals().await?;
    adapter.stop_scan().await?;
    debug!(
        peripheral_count = peripherals.len(),
        "scan collected peripherals"
    );
    let mut found = Vec::new();
    for peripheral in peripherals {
        if let Some(device) = matched_radiacode(&peripheral).await {
            found.push(device);
        }
    }
    let found = sort_and_dedup(found);
    info!(count = found.len(), "ble scan complete");
    Ok(found)
}

async fn readable_properties(peripheral: &Peripheral) -> Option<PeripheralProperties> {
    match peripheral.properties().await {
        Ok(props) => props,
        Err(error) => {
            debug!(
                address = %peripheral.address(),
                %error,
                "skipping peripheral with unavailable properties"
            );
            None
        }
    }
}

async fn matched_radiacode(peripheral: &Peripheral) -> Option<DiscoveredDevice> {
    let props = readable_properties(peripheral).await?;
    let advertisement = Advertisement {
        address: peripheral.address().to_string(),
        local_name: props.local_name,
        services: props.services,
        rssi: props.rssi,
    };
    if !is_radiacode_advertisement(&advertisement) {
        return None;
    }
    debug!(
        address = %advertisement.address,
        local_name = ?advertisement.local_name,
        rssi = ?advertisement.rssi,
        "matched radiacode advertisement"
    );
    Some(discovered_device(advertisement))
}
