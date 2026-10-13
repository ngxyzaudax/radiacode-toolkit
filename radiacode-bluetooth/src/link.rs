use std::time::Duration;

use btleplug::api::{Central, Peripheral as _};
use btleplug::platform::{Adapter, Peripheral};
use tokio::time::timeout;
use tracing::{debug, warn};

use crate::address::{address_matches, normalize_mac};

const STALE_DISCONNECT_COOLDOWN: Duration = Duration::from_millis(500);
const DISCONNECT_TIMEOUT: Duration = Duration::from_secs(2);

pub async fn disconnect_stale(peripheral: &Peripheral) {
    if !peripheral.is_connected().await.unwrap_or(false) {
        return;
    }
    let address = peripheral.address().to_string();
    debug!(%address, "disconnecting stale peripheral session");
    match timeout(DISCONNECT_TIMEOUT, peripheral.disconnect()).await {
        Ok(Ok(())) => {}
        Ok(Err(error)) => warn!(%address, %error, "stale disconnect failed"),
        Err(_) => warn!(%address, "stale disconnect timed out"),
    }
    tokio::time::sleep(STALE_DISCONNECT_COOLDOWN).await;
}

pub async fn disconnect_cached_peripheral(adapter: &Adapter, mac: &str) {
    let Ok(target) = normalize_mac(mac) else {
        return;
    };
    for peripheral in adapter.peripherals().await.unwrap_or_default() {
        if !address_matches(&peripheral.address().to_string(), &target) {
            continue;
        }
        if peripheral.is_connected().await.unwrap_or(false) {
            debug!(%target, "disconnecting cached peripheral before fresh scan");
            let _ = timeout(DISCONNECT_TIMEOUT, peripheral.disconnect()).await;
        }
    }
}
