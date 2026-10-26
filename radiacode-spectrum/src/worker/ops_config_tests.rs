use crate::test_device::VirtualDevice;
use radiacode_lib::protocol::{Command, VirtSfr};

use super::messages::WorkerEvent;
use super::ops_config::{
    handle_apply_device_config, handle_fetch_device_config, handle_sync_device_clock,
};
use super::ops_connect::handle_disconnect;
use super::ops_monitor::handle_monitor;
use super::ops_spectrum::{handle_dose_reset, handle_reset, handle_spectrum};
use super::test_fixtures::{
    Harness, configured_device, count_events, epoch_of, is_disconnected, is_error,
};

#[tokio::test(start_paused = true)]
async fn fetch_device_config_publishes_limits_and_config() {
    let mut harness = Harness::connected(configured_device()).await;
    handle_fetch_device_config(
        &harness.sender,
        &mut harness.session,
        &epoch_of(&harness.live_epoch),
    )
    .await;
    let events = harness.drain();
    assert!(matches!(events[0], WorkerEvent::AlarmLimits(_)));
    assert!(matches!(events[1], WorkerEvent::DeviceConfig(config) if config.brightness == 5));
    assert!(harness.session.alarm_limits.is_some());
}

#[tokio::test(start_paused = true)]
async fn apply_device_config_writes_and_reloads() {
    let mut harness = Harness::connected(configured_device()).await;
    let mut device = harness.session.device.take().expect("device");
    let config = radiacode_lib::config::DeviceConfig {
        brightness: 9,
        ..device.load_device_config().await.expect("config")
    };
    harness.session.device = Some(device);
    handle_apply_device_config(
        &harness.sender,
        &mut harness.session,
        config,
        &epoch_of(&harness.live_epoch),
    )
    .await;
    let events = harness.drain();
    assert!(matches!(events[1], WorkerEvent::DeviceConfig(loaded) if loaded.brightness == 9));
    assert_eq!(count_events(&events, is_error), 0);
    assert_eq!(harness.handle.device().register(VirtSfr::DispBrt), Some(9));
}

#[tokio::test(start_paused = true)]
async fn sync_device_clock_sets_time_and_resets_device_time() {
    let mut harness = Harness::connected(configured_device()).await;
    let set_time_before = harness.handle.device().request_count(Command::SetTime);
    handle_sync_device_clock(
        &harness.sender,
        &mut harness.session,
        &epoch_of(&harness.live_epoch),
    )
    .await;
    assert!(harness.drain().is_empty());
    let device = harness.handle.device();
    assert_eq!(device.request_count(Command::SetTime), set_time_before + 1);
    assert_eq!(device.register(VirtSfr::DeviceTime), Some(0));
}

#[tokio::test(start_paused = true)]
async fn disconnect_releases_device_and_notifies() {
    let mut harness = Harness::connected(configured_device()).await;
    handle_disconnect(&harness.sender, &mut harness.session).await;
    let events = harness.drain();
    assert_eq!(count_events(&events, is_disconnected), 1);
    assert!(harness.handle.is_disconnected());
    assert!(!harness.session.has_device());
}

#[tokio::test(start_paused = true)]
async fn operations_without_device_emit_nothing() {
    let mut harness = Harness::connected(VirtualDevice::default()).await;
    harness.session.device = None;
    let epoch = epoch_of(&harness.live_epoch);
    handle_monitor(&harness.sender, &mut harness.session, &epoch).await;
    handle_spectrum(&harness.sender, &mut harness.session, &epoch).await;
    handle_dose_reset(&harness.sender, &mut harness.session, &epoch).await;
    handle_reset(&harness.sender, &mut harness.session, &epoch).await;
    handle_fetch_device_config(&harness.sender, &mut harness.session, &epoch).await;
    handle_sync_device_clock(&harness.sender, &mut harness.session, &epoch).await;
    assert!(harness.drain().is_empty());
}
