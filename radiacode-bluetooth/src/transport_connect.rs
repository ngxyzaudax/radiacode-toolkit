use std::time::Duration;

use btleplug::api::{Characteristic, Peripheral as _};
use btleplug::platform::Peripheral;
use futures::StreamExt;
use tracing::{debug, info};

use crate::adapter::{find_peripheral, resolve_peripheral};
use crate::ble_error::BleError;
use crate::drain::{NotificationStream, drain_for_settle};
use crate::link::{disconnect_cached_peripheral, disconnect_stale};
use crate::scan_session::adapter_for_connect;
use crate::uuids;

const LINK_SETTLE: Duration = Duration::from_millis(250);
const RECONNECT_COOLDOWN: Duration = Duration::from_millis(500);
const FRESH_SCAN: Duration = Duration::from_secs(2);

pub struct OpenLink {
    pub peripheral: Peripheral,
    pub write_char: Characteristic,
    pub notify_char: Characteristic,
    pub notifications: NotificationStream,
}

pub async fn open_cached_link(mac: &str) -> std::result::Result<OpenLink, BleError> {
    info!(%mac, "ble transport connect");
    let adapter = adapter_for_connect().await?;
    let peripheral = resolve_peripheral(&adapter, mac).await?;
    open_link(peripheral).await
}

pub async fn open_fresh_link(mac: &str) -> std::result::Result<OpenLink, BleError> {
    info!(%mac, "ble transport fresh connect");
    let adapter = adapter_for_connect().await?;
    disconnect_cached_peripheral(&adapter, mac).await;
    tokio::time::sleep(RECONNECT_COOLDOWN).await;
    let peripheral = find_peripheral(&adapter, mac, FRESH_SCAN).await?;
    open_link(peripheral).await
}

async fn open_link(peripheral: Peripheral) -> std::result::Result<OpenLink, BleError> {
    let address = peripheral.address().to_string();
    disconnect_stale(&peripheral).await;
    debug!(%address, "connecting peripheral");
    peripheral.connect().await?;
    debug!(%address, "discovering services");
    peripheral.discover_services().await?;
    let write_char = find_characteristic(&peripheral, uuids::WRITE)?;
    let notify_char = find_characteristic(&peripheral, uuids::NOTIFY)?;
    let mut notifications = subscribe_notifications(&peripheral, &notify_char).await?;
    settle_link(&mut notifications).await;
    info!(%address, "ble transport ready");
    Ok(OpenLink {
        peripheral,
        write_char,
        notify_char,
        notifications,
    })
}

async fn subscribe_notifications(
    peripheral: &Peripheral,
    notify_char: &Characteristic,
) -> std::result::Result<NotificationStream, BleError> {
    let _ = peripheral.unsubscribe(notify_char).await;
    peripheral.subscribe(notify_char).await?;
    Ok(peripheral.notifications().await?.boxed())
}

async fn settle_link(notifications: &mut NotificationStream) {
    tokio::time::sleep(LINK_SETTLE).await;
    drain_for_settle(notifications).await;
}

fn find_characteristic(
    peripheral: &Peripheral,
    uuid: uuid::Uuid,
) -> std::result::Result<Characteristic, BleError> {
    peripheral
        .characteristics()
        .into_iter()
        .find(|characteristic| characteristic.uuid == uuid)
        .ok_or(BleError::CharacteristicMissing)
}
