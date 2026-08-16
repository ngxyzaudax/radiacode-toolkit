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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DrainMode {
    PeekThenQuiet,
    WaitForQuiet,
}

pub async fn drain_until_quiet(
    notifications: &mut BoxStream<'static, btleplug::api::ValueNotification>,
) {
    drain_for(notifications, MAX_DRAIN, DrainMode::WaitForQuiet).await;
}

pub async fn drain_for_settle(
    notifications: &mut BoxStream<'static, btleplug::api::ValueNotification>,
) {
    drain_for(notifications, SETTLE_DRAIN, DrainMode::WaitForQuiet).await;
}

async fn drain_for(
    notifications: &mut BoxStream<'static, btleplug::api::ValueNotification>,
    max_drain: Duration,
    mode: DrainMode,
) {
    let mut drained = 0usize;
    let mut last_received = Instant::now();
    if mode == DrainMode::PeekThenQuiet {
        match timeout(Duration::from_millis(1), notifications.next()).await {
            Ok(None) => return,
            Err(_) => return,
            Ok(Some(_)) => {
                drained += 1;
                last_received = Instant::now();
            }
        }
    }
    let deadline = Instant::now() + max_drain;
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
        debug!(drained, ?mode, "drained stale ble notifications");
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
    drain_for(notifications, COMMAND_DRAIN, DrainMode::PeekThenQuiet).await;

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

#[cfg(test)]
mod drain_mode_tests {
    use std::pin::Pin;
    use std::task::{Context, Poll};

    use std::time::Instant;

    use futures::Stream;
    use futures::StreamExt;
    use tokio::time::{Duration, sleep};

    use super::{DrainMode, QUIET_GAP, drain_for};

    type FutureNotification = std::pin::Pin<
        Box<dyn std::future::Future<Output = Option<btleplug::api::ValueNotification>> + Send>,
    >;

    struct TimedStream {
        pending: Option<FutureNotification>,
        done: bool,
    }

    impl Stream for TimedStream {
        type Item = btleplug::api::ValueNotification;

        fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            if self.done {
                return Poll::Ready(None);
            }
            if self.pending.is_none() {
                self.pending = Some(Box::pin(async {
                    sleep(Duration::from_millis(5)).await;
                    Some(btleplug::api::ValueNotification {
                        uuid: uuid::Uuid::nil(),
                        value: vec![0xAA],
                    })
                }));
            }
            let poll = self.pending.as_mut().unwrap().as_mut().poll(cx);
            if poll.is_ready() {
                self.done = true;
                self.pending = None;
            }
            poll
        }
    }

    struct EmptyStream;

    impl Stream for EmptyStream {
        type Item = btleplug::api::ValueNotification;

        fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            Poll::Ready(None)
        }
    }

    #[tokio::test]
    async fn peek_then_quiet_returns_immediately_on_idle_stream() {
        let mut stream: futures::stream::BoxStream<'static, btleplug::api::ValueNotification> =
            Box::pin(EmptyStream);
        let started = Instant::now();
        drain_for(
            &mut stream,
            Duration::from_secs(1),
            DrainMode::PeekThenQuiet,
        )
        .await;
        assert!(started.elapsed() < Duration::from_millis(50));
    }

    #[tokio::test]
    async fn wait_for_quiet_consumes_late_notification() {
        let mut stream: futures::stream::BoxStream<'static, btleplug::api::ValueNotification> =
            Box::pin(TimedStream {
                pending: None,
                done: false,
            });
        drain_for(
            &mut stream,
            QUIET_GAP + Duration::from_millis(200),
            DrainMode::WaitForQuiet,
        )
        .await;
        assert!(stream.next().await.is_none());
    }
}
