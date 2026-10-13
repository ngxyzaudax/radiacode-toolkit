use async_trait::async_trait;
use btleplug::api::Peripheral as _;
use radiacode_core::{RadiaCode, Result, SessionRestore};
use radiacode_protocol::{BytesBuffer, Transport};
use tracing::info;

use crate::ble_error::{map_ble_error, map_ble_protocol_error};
use crate::drain::drain_until_quiet;
use crate::execute::execute_request;
use crate::transport_connect::{OpenLink, open_cached_link, open_fresh_link};

pub struct BluetoothTransport {
    link: OpenLink,
}

impl BluetoothTransport {
    pub async fn connect(mac: &str) -> Result<Self> {
        let link = open_cached_link(mac).await.map_err(map_ble_error)?;
        Ok(Self { link })
    }

    pub async fn connect_fresh(mac: &str) -> Result<Self> {
        let link = open_fresh_link(mac).await.map_err(map_ble_error)?;
        Ok(Self { link })
    }
}

#[async_trait(?Send)]
impl Transport for BluetoothTransport {
    async fn execute(&mut self, request: &[u8]) -> radiacode_protocol::Result<BytesBuffer> {
        execute_request(
            &self.link.peripheral,
            &self.link.write_char,
            &mut self.link.notifications,
            request,
        )
        .await
    }

    async fn drain_link(&mut self) {
        drain_until_quiet(&mut self.link.notifications).await;
    }

    async fn disconnect(self: Box<Self>) -> radiacode_protocol::Result<()> {
        info!("ble transport disconnect");
        let _ = self
            .link
            .peripheral
            .unsubscribe(&self.link.notify_char)
            .await;
        self.link
            .peripheral
            .disconnect()
            .await
            .map_err(|error| map_ble_protocol_error(error.into()))?;
        Ok(())
    }

    async fn link_rssi_dbm(&self) -> Option<i16> {
        self.link
            .peripheral
            .properties()
            .await
            .ok()
            .flatten()
            .and_then(|props| props.rssi)
    }

    async fn sample_link_rssi_dbm(&self) -> Option<i16> {
        let address = self.link.peripheral.address().to_string();
        match crate::rssi::read_mgmt_rssi_dbm(&address).await {
            Some(rssi) => Some(rssi),
            None => self.link_rssi_dbm().await,
        }
    }
}

pub async fn connect(mac: &str) -> Result<RadiaCode> {
    RadiaCode::open(
        Box::new(BluetoothTransport::connect(mac).await?),
        false,
        None,
    )
    .await
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
