use async_trait::async_trait;
use radiacode_lib::bluetooth::BluetoothTransport;
use radiacode_lib::protocol::BytesBuffer;
use radiacode_lib::{Result, RssiStatus, Transport};

use super::{combine_rssi, read_mgmt_rssi};

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

    async fn link_rssi(&mut self) -> RssiStatus {
        self.inner.link_rssi().await
    }

    async fn sample_link_rssi(&mut self) -> RssiStatus {
        let address = self.inner.address();
        let cached = self.inner.link_rssi().await;
        combine_rssi(read_mgmt_rssi(&address).await, cached)
    }
}
