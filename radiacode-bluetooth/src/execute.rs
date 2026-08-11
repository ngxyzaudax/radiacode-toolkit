use std::time::Duration;

use btleplug::api::{Characteristic, Peripheral as _, WriteType};
use btleplug::platform::Peripheral;
use futures::StreamExt;
use futures::stream::BoxStream;
use radiacode_protocol::{
    BytesBuffer, Error, ResponseAssembler, Result, framed_request_header, response_matches_request,
};
use tokio::time::{Instant, timeout};
use tracing::{debug, warn};

use crate::ble_error::map_ble_protocol_error;
use crate::uuids::{CHUNK_SIZE, RESPONSE_TIMEOUT_SECS};

const QUIET_GAP: Duration = Duration::from_millis(120);
const MAX_DRAIN: Duration = Duration::from_millis(2500);
const COMMAND_DRAIN: Duration = Duration::from_millis(400);
const SETTLE_DRAIN: Duration = Duration::from_millis(500);

pub async fn drain_until_quiet(
    notifications: &mut BoxStream<'static, btleplug::api::ValueNotification>,
) {
    drain_for(notifications, MAX_DRAIN).await;
}

pub async fn drain_for_settle(
    notifications: &mut BoxStream<'static, btleplug::api::ValueNotification>,
) {
    drain_for(notifications, SETTLE_DRAIN).await;
}

async fn drain_for(
    notifications: &mut BoxStream<'static, btleplug::api::ValueNotification>,
    max_drain: Duration,
) {
    let deadline = Instant::now() + max_drain;
    let mut last_received = Instant::now();
    let mut drained = 0usize;
    while Instant::now() < deadline {
        let slice = deadline.saturating_duration_since(Instant::now());
        if slice.is_zero() {
            break;
        }
        let wait = slice.min(Duration::from_millis(25));
        match timeout(wait, notifications.next()).await {
            Ok(Some(_)) => {
                drained += 1;
                last_received = Instant::now();
            }
            Ok(None) => break,
            Err(_) if last_received.elapsed() >= QUIET_GAP => break,
            Err(_) => {}
        }
    }
    if drained > 0 {
        debug!(drained, "drained stale ble notifications");
    }
}

pub async fn execute_request(
    peripheral: &Peripheral,
    write_char: &Characteristic,
    notifications: &mut BoxStream<'static, btleplug::api::ValueNotification>,
    request: &[u8],
) -> Result<BytesBuffer> {
    let expected = framed_request_header(request)?;
    debug!(request_len = request.len(), "ble execute request");
    drain_for(notifications, COMMAND_DRAIN).await;

    for chunk in request.chunks(CHUNK_SIZE) {
        peripheral
            .write(write_char, chunk, WriteType::WithoutResponse)
            .await
            .map_err(|error| map_ble_protocol_error(error.into()))?;
    }

    let mut assembler = ResponseAssembler::default();
    let deadline = Instant::now() + Duration::from_secs(RESPONSE_TIMEOUT_SECS);
    let mut discarded = 0usize;

    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            warn!(
                request_len = request.len(),
                discarded, "ble response timed out"
            );
            drain_until_quiet(notifications).await;
            return Err(Error::Timeout);
        }

        let next = timeout(remaining, notifications.next())
            .await
            .map_err(|_| Error::Timeout)?
            .ok_or(Error::ConnectionClosed)?;

        if let Some(payload) = assembler.push(&next.value)? {
            if response_matches_request(&payload, expected) {
                debug!(
                    response_len = payload.len(),
                    discarded, "ble response complete"
                );
                return Ok(BytesBuffer::new(payload));
            }
            discarded += 1;
            warn!(
                response_len = payload.len(),
                discarded, "discarding unrelated ble frame"
            );
            assembler = ResponseAssembler::default();
        }
    }
}
