use std::time::Duration;

use btleplug::api::ValueNotification;
use futures::StreamExt;
use futures::stream::BoxStream;
use tokio::time::{Instant, timeout};
use tracing::debug;

pub type NotificationStream = BoxStream<'static, ValueNotification>;

pub(crate) const QUIET_GAP: Duration = Duration::from_millis(120);
const MAX_DRAIN: Duration = Duration::from_millis(2500);
const COMMAND_DRAIN: Duration = Duration::from_millis(400);
const SETTLE_DRAIN: Duration = Duration::from_millis(500);
const PEEK_WAIT: Duration = Duration::from_millis(1);
const POLL_SLICE: Duration = Duration::from_millis(25);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DrainMode {
    PeekThenQuiet,
    WaitForQuiet,
}

pub async fn drain_until_quiet(notifications: &mut NotificationStream) {
    drain_for(notifications, MAX_DRAIN, DrainMode::WaitForQuiet).await;
}

pub async fn drain_for_settle(notifications: &mut NotificationStream) {
    drain_for(notifications, SETTLE_DRAIN, DrainMode::WaitForQuiet).await;
}

pub async fn drain_before_command(notifications: &mut NotificationStream) {
    drain_for(notifications, COMMAND_DRAIN, DrainMode::PeekThenQuiet).await;
}

pub(crate) async fn drain_for(
    notifications: &mut NotificationStream,
    max_drain: Duration,
    mode: DrainMode,
) {
    let peeked = match mode {
        DrainMode::PeekThenQuiet => match peek_notification(notifications).await {
            Some(()) => 1,
            None => return,
        },
        DrainMode::WaitForQuiet => 0,
    };
    let drained = peeked + drain_until_gap(notifications, Instant::now() + max_drain).await;
    if drained > 0 {
        debug!(drained, ?mode, "drained stale ble notifications");
    }
}

async fn peek_notification(notifications: &mut NotificationStream) -> Option<()> {
    timeout(PEEK_WAIT, notifications.next())
        .await
        .ok()
        .flatten()
        .map(|_| ())
}

async fn drain_until_gap(notifications: &mut NotificationStream, deadline: Instant) -> usize {
    let mut drained = 0usize;
    let mut last_received = Instant::now();
    loop {
        let slice = deadline.saturating_duration_since(Instant::now());
        if slice.is_zero() {
            return drained;
        }
        match timeout(slice.min(POLL_SLICE), notifications.next()).await {
            Ok(Some(_)) => {
                drained += 1;
                last_received = Instant::now();
            }
            Ok(None) => return drained,
            Err(_) if last_received.elapsed() >= QUIET_GAP => return drained,
            Err(_) => {}
        }
    }
}
