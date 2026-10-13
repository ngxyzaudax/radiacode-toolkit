use std::time::Duration;

use btleplug::api::{Characteristic, Peripheral as _, WriteType};
use btleplug::platform::Peripheral;
use futures::StreamExt;
use radiacode_protocol::{
    BytesBuffer, Error, ResponseAssembler, Result, framed_request_header, response_matches_request,
};
use tokio::time::{Instant, timeout};
use tracing::{debug, warn};

use crate::ble_error::map_ble_protocol_error;
use crate::drain::{NotificationStream, drain_before_command, drain_until_quiet};
use crate::uuids::{CHUNK_SIZE, RESPONSE_TIMEOUT_SECS};

pub async fn execute_request(
    peripheral: &Peripheral,
    write_char: &Characteristic,
    notifications: &mut NotificationStream,
    request: &[u8],
) -> Result<BytesBuffer> {
    let expected = framed_request_header(request)?;
    debug!(request_len = request.len(), "ble execute request");
    drain_before_command(notifications).await;
    write_chunks(peripheral, write_char, request).await?;
    let deadline = Instant::now() + Duration::from_secs(RESPONSE_TIMEOUT_SECS);
    match await_matching_response(notifications, expected, deadline).await {
        Err(Error::Timeout) => {
            warn!(request_len = request.len(), "ble response timed out");
            drain_until_quiet(notifications).await;
            Err(Error::Timeout)
        }
        outcome => outcome,
    }
}

async fn write_chunks(
    peripheral: &Peripheral,
    write_char: &Characteristic,
    request: &[u8],
) -> Result<()> {
    for chunk in request.chunks(CHUNK_SIZE) {
        peripheral
            .write(write_char, chunk, WriteType::WithoutResponse)
            .await
            .map_err(|error| map_ble_protocol_error(error.into()))?;
    }
    Ok(())
}

pub(crate) async fn await_matching_response(
    notifications: &mut NotificationStream,
    expected: [u8; 4],
    deadline: Instant,
) -> Result<BytesBuffer> {
    let mut assembler = ResponseAssembler::default();
    let mut discarded = 0usize;
    loop {
        let chunk = next_chunk_before(notifications, deadline).await?;
        let Some(payload) = assembler.push(&chunk)? else {
            continue;
        };
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

async fn next_chunk_before(
    notifications: &mut NotificationStream,
    deadline: Instant,
) -> Result<Vec<u8>> {
    let remaining = deadline.saturating_duration_since(Instant::now());
    if remaining.is_zero() {
        return Err(Error::Timeout);
    }
    timeout(remaining, notifications.next())
        .await
        .map_err(|_| Error::Timeout)?
        .map(|notification| notification.value)
        .ok_or(Error::ConnectionClosed)
}
