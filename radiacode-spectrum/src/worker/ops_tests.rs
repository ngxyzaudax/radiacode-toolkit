use radiacode_core::test_support::encode::u32_words;
use radiacode_core::test_support::{ScriptedReply, VirtualDevice};
use radiacode_core::{Command, VirtSfr};

use super::epoch::end_session;
use super::messages::WorkerEvent;
use super::ops_config::{
    handle_apply_device_config, handle_fetch_device_config, handle_sync_device_clock,
};
use super::ops_connect::handle_disconnect;
use super::ops_monitor::handle_monitor;
use super::ops_spectrum::{handle_dose_reset, handle_reset, handle_spectrum};
use super::test_fixtures::{Harness, configured_device, count_events, epoch_of, usb_endpoint};

fn is_monitor_sample(event: &WorkerEvent) -> bool {
    matches!(event, WorkerEvent::MonitorSample(_))
}

fn is_spectrum(event: &WorkerEvent) -> bool {
    matches!(event, WorkerEvent::Spectrum(_))
}

fn is_error(event: &WorkerEvent) -> bool {
    matches!(event, WorkerEvent::Error(_))
}

fn is_disconnected(event: &WorkerEvent) -> bool {
    matches!(event, WorkerEvent::Disconnected)
}

#[tokio::test(start_paused = true)]
async fn monitor_poll_publishes_limits_sample_status_and_completion() {
    let mut harness = Harness::connected(configured_device()).await;
    handle_monitor(
        &harness.sender,
        &mut harness.session,
        &epoch_of(&harness.live_epoch),
    )
    .await;
    let events = harness.drain();
    assert!(matches!(events[0], WorkerEvent::AlarmLimits(limits) if limits.l2_count_rate == 60.0));
    assert_eq!(count_events(&events, is_monitor_sample), 1);
    assert!(matches!(
        events[2],
        WorkerEvent::DeviceStatus(status) if status.battery_percent == Some(87.5)
    ));
    assert!(matches!(events[3], WorkerEvent::MonitorPollComplete));
    assert_eq!(harness.session.monitor_polls, 1);
    assert!(harness.session.alarm_limits.is_some());
    assert!(harness.session.has_device());
}

#[tokio::test(start_paused = true)]
async fn monitor_poll_without_data_reports_status_only() {
    let device = configured_device().with_string(radiacode_core::VirtString::DataBuf, &[]);
    let mut harness = Harness::connected(device).await;
    handle_monitor(
        &harness.sender,
        &mut harness.session,
        &epoch_of(&harness.live_epoch),
    )
    .await;
    let events = harness.drain();
    assert_eq!(count_events(&events, is_monitor_sample), 0);
    assert!(
        events
            .iter()
            .any(|event| matches!(event, WorkerEvent::MonitorPollComplete))
    );
    assert_eq!(harness.session.monitor_polls, 0);
}

#[tokio::test(start_paused = true)]
async fn monitor_poll_after_session_end_is_silent() {
    let mut harness = Harness::connected(configured_device()).await;
    let epoch = epoch_of(&harness.live_epoch);
    end_session(&harness.live_epoch);
    handle_monitor(&harness.sender, &mut harness.session, &epoch).await;
    let events = harness.drain();
    assert_eq!(count_events(&events, is_monitor_sample), 0);
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, WorkerEvent::MonitorPollComplete))
    );
    assert!(harness.session.has_device());
}

#[tokio::test(start_paused = true)]
async fn spectrum_fetch_retries_transient_errors() {
    let mut harness = Harness::connected(configured_device()).await;
    harness.script(Command::RdVirtString, ScriptedReply::timeout());
    handle_spectrum(
        &harness.sender,
        &mut harness.session,
        &epoch_of(&harness.live_epoch),
    )
    .await;
    let events = harness.drain();
    assert_eq!(count_events(&events, is_spectrum), 1);
    let WorkerEvent::Spectrum(view) = &events[0] else {
        panic!("unexpected event {:?}", events[0]);
    };
    assert_eq!(view.total_counts, 18);
    assert_eq!(view.counts, vec![5, 6, 7]);
}

#[tokio::test(start_paused = true)]
async fn spectrum_fetch_reports_error_without_reconnect_when_endpoint_unknown() {
    let mut harness = Harness::connected(configured_device()).await;
    for _ in 0..3 {
        harness.script(Command::RdVirtString, ScriptedReply::timeout());
    }
    handle_spectrum(
        &harness.sender,
        &mut harness.session,
        &epoch_of(&harness.live_epoch),
    )
    .await;
    let events = harness.drain();
    assert_eq!(count_events(&events, is_error), 1);
    assert!(harness.session.has_device());
}

#[tokio::test(start_paused = true)]
async fn connection_loss_without_restore_cache_disconnects() {
    let mut harness = Harness::connected(configured_device()).await;
    harness.script(Command::RdVirtString, ScriptedReply::connection_closed());
    harness.session.session_endpoint = Some(usb_endpoint());
    harness.session.session_restore = None;
    handle_spectrum(
        &harness.sender,
        &mut harness.session,
        &epoch_of(&harness.live_epoch),
    )
    .await;
    let events = harness.drain();
    assert_eq!(count_events(&events, is_disconnected), 1);
    assert!(!harness.session.has_device());
}

#[tokio::test(start_paused = true)]
async fn connection_loss_after_session_end_drops_device_silently() {
    let mut harness = Harness::connected(configured_device()).await;
    harness.script(Command::RdVirtString, ScriptedReply::connection_closed());
    harness.session.session_endpoint = Some(usb_endpoint());
    let epoch = epoch_of(&harness.live_epoch);
    end_session(&harness.live_epoch);
    handle_spectrum(&harness.sender, &mut harness.session, &epoch).await;
    assert!(harness.drain().is_empty());
    assert!(!harness.session.has_device());
}

#[tokio::test(start_paused = true)]
async fn dose_reset_writes_register_and_confirms() {
    let mut harness = Harness::connected(configured_device()).await;
    let writes_before = harness.handle.device().request_count(Command::WrVirtSfr);
    handle_dose_reset(
        &harness.sender,
        &mut harness.session,
        &epoch_of(&harness.live_epoch),
    )
    .await;
    let events = harness.drain();
    assert!(matches!(events[0], WorkerEvent::DoseResetComplete));
    let writes = harness.handle.device().request_count(Command::WrVirtSfr);
    assert_eq!(writes, writes_before + 1);
}

#[tokio::test(start_paused = true)]
async fn spectrum_reset_clears_then_refetches() {
    let mut harness = Harness::connected(configured_device()).await;
    harness.script(
        Command::WrVirtString,
        ScriptedReply::Payload(u32_words(&[1])),
    );
    handle_reset(
        &harness.sender,
        &mut harness.session,
        &epoch_of(&harness.live_epoch),
    )
    .await;
    let events = harness.drain();
    assert_eq!(count_events(&events, is_spectrum), 1);
    assert_eq!(
        harness.handle.device().request_count(Command::WrVirtString),
        1
    );
}

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
    let config = radiacode_core::DeviceConfig {
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
