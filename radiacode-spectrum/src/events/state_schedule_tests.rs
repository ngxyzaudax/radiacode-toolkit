use std::time::{Duration, Instant};

use radiacode_core::DeviceEndpoint;

use super::test_fixtures::{device_info, isolated_state, monitor_sample, spectrum_view};
use crate::model::ConnectionState;
use crate::worker::{WorkerCommand, WorkerEvent};

#[test]
fn dose_reset_requests_monitor_refresh() {
    let (mut state, _dir) = isolated_state();
    state.apply_event(WorkerEvent::Connected(device_info()), true);
    state.apply_event(WorkerEvent::MonitorSample(monitor_sample(12.0)), true);
    let command = state.apply_event(WorkerEvent::DoseResetComplete, true);
    assert!(matches!(command, Some(WorkerCommand::FetchMonitor)));
    assert!(state.dosimeter.latest.is_none());
    assert_eq!(state.status, "Dose reset.");
}

#[test]
fn error_while_connecting_returns_to_disconnected() {
    let (mut state, _dir) = isolated_state();
    state.connection = ConnectionState::Connecting;
    state.connecting_endpoint = Some(DeviceEndpoint::Usb { serial: "X".into() });
    state.scanning = true;
    state.apply_event(WorkerEvent::Error("device not found".into()), true);
    assert_eq!(state.connection, ConnectionState::Disconnected);
    assert!(state.connecting_endpoint.is_none());
    assert!(!state.scanning);
    assert_eq!(state.status, "device not found");
}

#[test]
fn usb_permission_event_marks_endpoint_for_prompt() {
    let (mut state, _dir) = isolated_state();
    let endpoint = DeviceEndpoint::Usb { serial: "X".into() };
    state.apply_event(
        WorkerEvent::UsbPermissionRequired {
            endpoint: endpoint.clone(),
        },
        true,
    );
    assert_eq!(state.connecting_endpoint, Some(endpoint));
    assert_eq!(state.status, "USB access required.");
}

#[test]
fn reconnecting_drops_spectrum_but_keeps_connection() {
    let (mut state, _dir) = isolated_state();
    state.apply_event(WorkerEvent::Connected(device_info()), true);
    state.apply_event(WorkerEvent::Spectrum(spectrum_view()), true);
    state.apply_event(WorkerEvent::Reconnecting, true);
    assert!(state.spectrum.is_none());
    assert!(state.last_fetch.is_none());
    assert_eq!(state.connection, ConnectionState::Connected);
    assert_eq!(state.status, "Connection lost, reconnecting…");
}

#[test]
fn busy_and_alarm_limit_events_apply_directly() {
    let (mut state, _dir) = isolated_state();
    state.apply_event(WorkerEvent::Busy(true), false);
    assert!(state.busy);
    let limits = crate::worker::test_fixtures::alarm_limits();
    state.apply_event(WorkerEvent::AlarmLimits(limits), true);
    assert_eq!(state.monitor.limits, Some(limits));
    assert_eq!(state.dosimeter.limits, Some(limits));
    let config = crate::worker::test_fixtures::device_config();
    assert!(
        state
            .apply_event(WorkerEvent::DeviceConfig(config), true)
            .is_none()
    );
}

#[test]
fn live_refresh_due_requires_connection_and_elapsed_interval() {
    let (mut state, _dir) = isolated_state();
    assert!(!state.live_refresh_due(true, 5));
    state.connection = ConnectionState::Connected;
    assert!(state.live_refresh_due(true, 5));
    assert!(!state.live_refresh_due(false, 5));
    state.last_fetch = Some(Instant::now());
    assert!(!state.live_refresh_due(true, 5));
    state.last_fetch = Some(Instant::now() - Duration::from_secs(6));
    assert!(state.live_refresh_due(true, 5));
    state.spectrum_fetch_pending = true;
    assert!(!state.live_refresh_due(true, 5));
}

#[test]
fn scheduling_is_idempotent_until_fetch_completes() {
    let (mut state, _dir) = isolated_state();
    assert!(state.try_schedule_spectrum());
    assert!(!state.try_schedule_spectrum());
    assert!(state.try_schedule_monitor());
    assert!(!state.try_schedule_monitor());
    state.apply_event(WorkerEvent::MonitorPollComplete, true);
    assert!(state.try_schedule_monitor());
}
