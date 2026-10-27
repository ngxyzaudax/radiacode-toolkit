use radiacode_lib::Error;

use super::epoch;
use super::messages::WorkerEvent;
use super::ops_connect::{connect_failure_event, handle_disconnect};
use super::ops_reconnect::handle_device_error;
use super::test_fixtures::{
    Harness, configured_device, count_events, epoch_of, is_disconnected, is_error, usb_endpoint,
};

#[test]
fn usb_permission_failure_maps_to_permission_event() {
    let endpoint = usb_endpoint();
    let event = connect_failure_event(&endpoint, &Error::UsbPermissionDenied);
    assert!(matches!(
        event,
        WorkerEvent::UsbPermissionRequired { endpoint: reported } if reported == endpoint
    ));
}

#[test]
fn other_connect_failures_map_to_error_event() {
    let event = connect_failure_event(&usb_endpoint(), &Error::DeviceNotFound);
    assert!(matches!(event, WorkerEvent::Error(message) if message == "device not found"));
}

#[tokio::test(start_paused = true)]
async fn disconnect_releases_device_and_reports_disconnected() {
    let mut harness = Harness::connected(configured_device()).await;
    handle_disconnect(&harness.sender, &mut harness.session).await;
    let events = harness.drain();
    assert!(!harness.session.has_device());
    assert_eq!(count_events(&events, is_disconnected), 1);
    assert_eq!(count_events(&events, is_error), 0);
}

#[tokio::test(start_paused = true)]
async fn disconnect_without_device_still_reports_disconnected() {
    let mut harness = Harness::connected(configured_device()).await;
    harness.session.device = None;
    handle_disconnect(&harness.sender, &mut harness.session).await;
    assert_eq!(count_events(&harness.drain(), is_disconnected), 1);
}

#[tokio::test(start_paused = true)]
async fn non_link_error_is_reported_and_keeps_device() {
    let mut harness = Harness::connected(configured_device()).await;
    let device = harness.session.device.take().expect("device");
    let epoch = epoch_of(&harness.live_epoch);
    let kept = handle_device_error(
        &harness.sender,
        &mut harness.session,
        device,
        Error::MonitorDataPending,
        &epoch,
    )
    .await;
    assert!(kept.is_some());
    let events = harness.drain();
    assert_eq!(count_events(&events, is_error), 1);
    assert_eq!(count_events(&events, is_disconnected), 0);
}

#[tokio::test(start_paused = true)]
async fn link_loss_without_session_endpoint_is_not_reconnected() {
    let mut harness = Harness::connected(configured_device()).await;
    harness.session.session_endpoint = None;
    let device = harness.session.device.take().expect("device");
    let epoch = epoch_of(&harness.live_epoch);
    let kept = handle_device_error(
        &harness.sender,
        &mut harness.session,
        device,
        Error::ConnectionClosed,
        &epoch,
    )
    .await;
    assert!(kept.is_some());
    assert_eq!(count_events(&harness.drain(), is_error), 1);
}

#[tokio::test(start_paused = true)]
async fn link_loss_after_session_end_drops_device_silently() {
    let mut harness = Harness::connected(configured_device()).await;
    harness.session.session_endpoint = Some(usb_endpoint());
    let device = harness.session.device.take().expect("device");
    let epoch = epoch_of(&harness.live_epoch);
    epoch::end_session(&harness.live_epoch);
    let kept = handle_device_error(
        &harness.sender,
        &mut harness.session,
        device,
        Error::ConnectionClosed,
        &epoch,
    )
    .await;
    assert!(kept.is_none());
    assert!(harness.drain().is_empty());
}

#[tokio::test(start_paused = true)]
async fn link_loss_without_cached_session_reports_disconnected() {
    let mut harness = Harness::connected(configured_device()).await;
    harness.session.session_endpoint = Some(usb_endpoint());
    harness.session.session_restore = None;
    let device = harness.session.device.take().expect("device");
    let epoch = epoch_of(&harness.live_epoch);
    let kept = handle_device_error(
        &harness.sender,
        &mut harness.session,
        device,
        Error::Timeout,
        &epoch,
    )
    .await;
    assert!(kept.is_none());
    let events = harness.drain();
    assert_eq!(count_events(&events, is_disconnected), 1);
    assert_eq!(count_events(&events, is_error), 0);
}
