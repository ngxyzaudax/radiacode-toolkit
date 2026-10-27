use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use async_trait::async_trait;
use radiacode_lib::protocol::BytesBuffer;
use radiacode_lib::{BoxedTransport, Result, RssiStatus, Transport};

use super::dispatch::respond;
use super::virtual_device::{RecordedRequest, ScriptedReply, VirtualDevice};

#[derive(Clone)]
pub struct DeviceHandle {
    device: Arc<Mutex<VirtualDevice>>,
    disconnected: Arc<AtomicBool>,
    drain_count: Arc<AtomicUsize>,
}

impl DeviceHandle {
    pub fn new(device: VirtualDevice) -> Self {
        Self {
            device: Arc::new(Mutex::new(device)),
            disconnected: Arc::new(AtomicBool::new(false)),
            drain_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn device(&self) -> MutexGuard<'_, VirtualDevice> {
        self.device.lock().unwrap()
    }

    pub fn is_disconnected(&self) -> bool {
        self.disconnected.load(Ordering::SeqCst)
    }

    pub fn drain_count(&self) -> usize {
        self.drain_count.load(Ordering::SeqCst)
    }

    pub fn transport(&self, rssi_dbm: Option<i16>) -> BoxedTransport {
        Box::new(FakeTransport {
            handle: self.clone(),
            rssi_dbm,
        })
    }
}

pub struct FakeTransport {
    handle: DeviceHandle,
    rssi_dbm: Option<i16>,
}

fn parse_framed_request(request: &[u8]) -> RecordedRequest {
    RecordedRequest {
        command: u16::from_le_bytes([request[4], request[5]]),
        seq: request[7],
        args: request[8..].to_vec(),
    }
}

fn scripted_or_dispatched(
    device: &mut VirtualDevice,
    request: &RecordedRequest,
) -> Result<ScriptedReply> {
    match device.take_script(request.command) {
        Some(reply) => Ok(reply),
        None => respond(device, request.command, &request.args).map(ScriptedReply::Payload),
    }
}

#[async_trait]
impl Transport for FakeTransport {
    async fn execute(&mut self, request: &[u8]) -> Result<BytesBuffer> {
        let parsed = parse_framed_request(request);
        let header = &request[4..8];
        let mut device = self.handle.device();
        device.requests.push(parsed.clone());
        match scripted_or_dispatched(&mut device, &parsed)? {
            ScriptedReply::Payload(payload) => Ok(BytesBuffer::new([header, &payload].concat())),
            ScriptedReply::RawResponse(raw) => Ok(BytesBuffer::new(raw)),
            ScriptedReply::Failure(error) => Err(error),
        }
    }

    async fn drain_link(&mut self) {
        self.handle.drain_count.fetch_add(1, Ordering::SeqCst);
    }

    async fn disconnect(self: Box<Self>) -> Result<()> {
        self.handle.disconnected.store(true, Ordering::SeqCst);
        Ok(())
    }

    async fn link_rssi(&mut self) -> RssiStatus {
        self.rssi_dbm.map(RssiStatus::Measured).unwrap_or_default()
    }

    async fn sample_link_rssi(&mut self) -> RssiStatus {
        self.rssi_dbm
            .map(|value| RssiStatus::Measured(value - 1))
            .unwrap_or_default()
    }
}
