use std::time::Duration;

use btleplug::api::{Central, Peripheral as _, PeripheralProperties, ScanFilter};
use btleplug::platform::Peripheral;
use radiacode_core::{
    DeviceEndpoint, DiscoveredDevice, model_from_advertisement, serial_from_advertisement,
};
use tracing::{debug, info};

use crate::adapter::default_adapter;
use crate::ble_error::BleError;
use crate::uuids;

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
        let Some(props) = readable_properties(&peripheral).await else {
            continue;
        };
        if let Some(device) = matched_radiacode(&peripheral, props) {
            found.push(device);
        }
    }
    found.sort_by(|left, right| {
        left.endpoint
            .address_label()
            .cmp(right.endpoint.address_label())
    });
    found.dedup_by(|left, right| left.endpoint == right.endpoint);
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

fn advertises_radiacode(props: &PeripheralProperties) -> bool {
    let advertises_service = props.services.contains(&uuids::SERVICE);
    let name_matches = props
        .local_name
        .as_deref()
        .is_some_and(|n| n.to_ascii_lowercase().contains("radiacode"));
    advertises_service || name_matches
}

fn matched_radiacode(
    peripheral: &Peripheral,
    props: PeripheralProperties,
) -> Option<DiscoveredDevice> {
    if !advertises_radiacode(&props) {
        return None;
    }
    let local_name = props.local_name.clone();
    let serial = local_name.as_deref().and_then(serial_from_advertisement);
    let model = local_name.as_deref().and_then(model_from_advertisement);
    let address = peripheral.address().to_string();
    debug!(
        %address,
        ?local_name,
        rssi = ?props.rssi,
        "matched radiacode advertisement"
    );
    let label = model
        .clone()
        .or_else(|| serial.clone())
        .or(local_name)
        .unwrap_or_else(|| "RadiaCode".into());
    Some(DiscoveredDevice {
        endpoint: DeviceEndpoint::Bluetooth { address },
        label,
        serial,
        model,
        rssi: props.rssi,
    })
}
