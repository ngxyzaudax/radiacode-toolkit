use std::time::Duration;

use async_trait::async_trait;
use btleplug::api::{Characteristic, Peripheral as _};
use btleplug::platform::Peripheral;
use futures::StreamExt;
use radiacode_core::{RadiaCode, Result, SessionRestore};
use radiacode_protocol::{BytesBuffer, Transport};
use tracing::{debug, info};

use crate::adapter::{find_peripheral, resolve_peripheral};
use crate::ble_error::{map_ble_error, map_ble_protocol_error, BleError};
use crate::execute::{drain_for_settle, drain_until_quiet, execute_request};
use crate::link::{disconnect_cached_peripheral, disconnect_stale};
use crate::scan_session::adapter_for_connect;
use crate::uuids;

const LINK_SETTLE: Duration = Duration::from_millis(250);
const RECONNECT_COOLDOWN: Duration = Duration::from_millis(500);
const FRESH_SCAN: Duration = Duration::from_secs(2);

pub struct BluetoothTransport {
    peripheral: Peripheral,
    write_char: Characteristic,
    notify_char: Characteristic,
    notifications: futures::stream::BoxStream<'static, btleplug::api::ValueNotification>,
}

impl BluetoothTransport {
    pub async fn connect(mac: &str) -> Result<Self> {
        info!(%mac, "ble transport connect");
        let adapter = adapter_for_connect().await.map_err(map_ble_error)?;
        let peripheral = resolve_peripheral(&adapter, mac).await.map_err(map_ble_error)?;
        Self::connect_peripheral(peripheral).await.map_err(map_ble_error)
    }

    pub async fn connect_fresh(mac: &str) -> Result<Self> {
        info!(%mac, "ble transport fresh connect");
        let adapter = adapter_for_connect().await.map_err(map_ble_error)?;
        disconnect_cached_peripheral(&adapter, mac).await;
        tokio::time::sleep(RECONNECT_COOLDOWN).await;
        let peripheral = find_peripheral(&adapter, mac, FRESH_SCAN)
            .await
            .map_err(map_ble_error)?;
        Self::connect_peripheral(peripheral).await.map_err(map_ble_error)
    }

    async fn connect_peripheral(peripheral: Peripheral) -> std::result::Result<Self, BleError> {
        let address = peripheral.address().to_string();
        disconnect_stale(&peripheral).await;
        debug!(%address, "connecting peripheral");
        peripheral.connect().await?;
        debug!(%address, "discovering services");
        peripheral.discover_services().await?;

        let write_char = find_characteristic(&peripheral, uuids::WRITE)?;
        let notify_char = find_characteristic(&peripheral, uuids::NOTIFY)?;
        let _ = peripheral.unsubscribe(&notify_char).await;
        peripheral.subscribe(&notify_char).await?;
        let notifications = peripheral.notifications().await?.boxed();
        let mut transport = Self {
            peripheral,
            write_char,
            notify_char,
            notifications,
        };
        transport.settle_link().await;
        info!(%address, "ble transport ready");
        Ok(transport)
    }

    async fn settle_link(&mut self) {
        tokio::time::sleep(LINK_SETTLE).await;
        drain_for_settle(&mut self.notifications).await;
    }
}

#[async_trait(?Send)]
impl Transport for BluetoothTransport {
    async fn execute(&mut self, request: &[u8]) -> radiacode_protocol::Result<BytesBuffer> {
        execute_request(
            &self.peripheral,
            &self.write_char,
            &mut self.notifications,
            request,
        )
        .await
    }

    async fn drain_link(&mut self) {
        drain_until_quiet(&mut self.notifications).await;
    }

    async fn disconnect(self: Box<Self>) -> radiacode_protocol::Result<()> {
        info!("ble transport disconnect");
        let _ = self.peripheral.unsubscribe(&self.notify_char).await;
        self.peripheral
            .disconnect()
            .await
            .map_err(|error| map_ble_protocol_error(error.into()))?;
        Ok(())
    }

    async fn link_rssi_dbm(&self) -> Option<i16> {
        self.peripheral
            .properties()
            .await
            .ok()
            .flatten()
            .and_then(|props| props.rssi)
    }

    async fn sample_link_rssi_dbm(&self) -> Option<i16> {
        let address = self.peripheral.address().to_string();
        if let Some(rssi) = crate::rssi::read_mgmt_rssi_dbm(&address).await {
            return Some(rssi);
        }
        self.link_rssi_dbm().await
    }
}

pub async fn connect(mac: &str) -> Result<RadiaCode> {
    RadiaCode::open(Box::new(BluetoothTransport::connect(mac).await?), false, None).await
}

pub async fn reconnect_session(mac: &str, restore: &SessionRestore) -> Result<RadiaCode> {
    info!(%mac, "radiacode bluetooth reconnect with cached session");
    RadiaCode::open(
        Box::new(BluetoothTransport::connect_fresh(mac).await?),
        false,
        Some(restore),
    )
    .await
}

fn find_characteristic(
    peripheral: &Peripheral,
    uuid: uuid::Uuid,
) -> std::result::Result<Characteristic, BleError> {
    peripheral
        .characteristics()
        .into_iter()
        .find(|c| c.uuid == uuid)
        .ok_or(BleError::CharacteristicMissing)
}
