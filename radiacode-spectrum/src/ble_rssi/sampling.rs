use async_trait::async_trait;
use radiacode_lib::bluetooth::BluetoothTransport;
use radiacode_lib::protocol::BytesBuffer;
use radiacode_lib::{Result, Transport};

use super::read_mgmt_rssi_dbm;

/// Bluetooth transport that samples RSSI through BlueZ before falling back to btleplug.
pub struct SamplingBluetoothTransport {
    inner: BluetoothTransport,
}

impl SamplingBluetoothTransport {
    pub fn new(inner: BluetoothTransport) -> Self {
        Self { inner }
    }
}

#[async_trait]
impl Transport for SamplingBluetoothTransport {
    async fn execute(&mut self, request: &[u8]) -> Result<BytesBuffer> {
        self.inner.execute(request).await
    }

    async fn drain_link(&mut self) {
        self.inner.drain_link().await;
    }

    async fn disconnect(self: Box<Self>) -> Result<()> {
        Box::new(self.inner).disconnect().await
    }

    async fn link_rssi_dbm(&mut self) -> Option<i16> {
        self.inner.link_rssi_dbm().await
    }

    async fn sample_link_rssi_dbm(&mut self) -> Option<i16> {
        let address = self.inner.address();
        match read_mgmt_rssi_dbm(&address).await {
            Some(rssi) => Some(rssi),
            None => self.inner.link_rssi_dbm().await,
        }
    }
}
