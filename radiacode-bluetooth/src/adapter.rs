use std::time::Duration;

use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::{Adapter, Manager, Peripheral};
use tracing::debug;

use crate::address::{address_matches, normalize_mac};
use crate::ble_error::BleError;

pub const CONNECT_DISCOVERY: Duration = Duration::from_millis(800);

pub async fn default_adapter() -> std::result::Result<Adapter, BleError> {
    let manager = Manager::new().await?;
    manager
        .adapters()
        .await?
        .into_iter()
        .next()
        .ok_or(BleError::AdapterNotFound)
}

pub async fn resolve_peripheral(
    adapter: &Adapter,
    mac: &str,
) -> std::result::Result<Peripheral, BleError> {
    let target = normalize_mac(mac)?;
    for peripheral in adapter.peripherals().await? {
        if address_matches(&peripheral.address().to_string(), &target) {
            debug!(%target, "resolved peripheral from known list");
            return Ok(peripheral);
        }
    }
    debug!(%target, "peripheral not cached, scanning");
    find_peripheral(adapter, mac, CONNECT_DISCOVERY).await
}

pub async fn find_peripheral(
    adapter: &Adapter,
    mac: &str,
    duration: Duration,
) -> std::result::Result<Peripheral, BleError> {
    let target = normalize_mac(mac)?;
    adapter.start_scan(ScanFilter::default()).await?;
    tokio::time::sleep(duration).await;

    let peripherals = adapter.peripherals().await?;
    adapter.stop_scan().await?;

    peripherals
        .into_iter()
        .find(|peripheral| address_matches(&peripheral.address().to_string(), &target))
        .ok_or(BleError::DeviceNotFound)
}
