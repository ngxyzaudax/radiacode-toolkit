use std::time::Instant;

use futures::StreamExt;
use futures::stream;
use tokio::time::Duration;

use crate::drain::{DrainMode, NotificationStream, QUIET_GAP, drain_for};
use crate::test_streams::{delayed_notification, notification};

fn empty_stream() -> NotificationStream {
    stream::empty().boxed()
}

#[tokio::test]
async fn peek_then_quiet_returns_immediately_on_idle_stream() {
    let mut notifications = empty_stream();
    let started = Instant::now();
    drain_for(
        &mut notifications,
        Duration::from_secs(1),
        DrainMode::PeekThenQuiet,
    )
    .await;
    assert!(started.elapsed() < Duration::from_millis(50));
}

#[tokio::test]
async fn wait_for_quiet_consumes_late_notification() {
    let mut notifications: NotificationStream =
        stream::once(delayed_notification(Duration::from_millis(5), vec![0xAA])).boxed();
    drain_for(
        &mut notifications,
        QUIET_GAP + Duration::from_millis(200),
        DrainMode::WaitForQuiet,
    )
    .await;
    assert!(notifications.next().await.is_none());
}

#[tokio::test]
async fn peek_then_quiet_drains_immediate_backlog() {
    let mut notifications: NotificationStream =
        stream::iter(vec![notification(vec![1]), notification(vec![2])])
            .chain(stream::pending())
            .boxed();
    drain_for(
        &mut notifications,
        QUIET_GAP + Duration::from_millis(100),
        DrainMode::PeekThenQuiet,
    )
    .await;
    assert!(
        tokio::time::timeout(Duration::from_millis(20), notifications.next())
            .await
            .is_err()
    );
}
