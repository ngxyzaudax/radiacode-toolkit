use radiacode_core::DeviceStatus;

use super::test_fixtures::{
    device_info, discovered, isolated_state, monitor_sample, spectrum_view,
};
use crate::model::ConnectionState;
use crate::worker::{WorkerCommand, WorkerEvent};

#[test]
fn scan_results_update_devices_and_status() {
    let (mut state, _dir) = isolated_state();
    state.scanning = true;
    assert!(
        state
            .apply_event(WorkerEvent::ScanFinished(Vec::new()), true)
            .is_none()
    );
    assert!(!state.scanning);
    assert!(state.scanned_once);
    assert_eq!(state.status, "No RadiaCode devices found.");
    state.apply_event(
        WorkerEvent::ScanFinished(vec![discovered("A"), discovered("B")]),
        true,
    );
    assert_eq!(state.devices.len(), 2);
    assert_eq!(state.status, "2 device(s) available.");
}

#[test]
fn connected_event_requests_monitor_fetch_and_tracks_session() {
    let (mut state, _dir) = isolated_state();
    state.connection = ConnectionState::Connecting;
    let command = state.apply_event(WorkerEvent::Connected(device_info()), true);
    assert!(matches!(command, Some(WorkerCommand::FetchMonitor)));
    assert_eq!(state.connection, ConnectionState::Connected);
    assert!(state.connecting_endpoint.is_none());
    assert_eq!(
        state.device_info.as_ref().map(|i| i.serial.as_str()),
        Some("RC-103-000001")
    );
    assert_eq!(state.status, "Connected. Acquiring monitor data…");
    assert_eq!(state.monitor.status, "Loading monitor data…");
}

#[test]
fn session_events_are_ignored_when_session_blocked() {
    let (mut state, _dir) = isolated_state();
    assert!(
        state
            .apply_event(WorkerEvent::Connected(device_info()), false)
            .is_none()
    );
    assert_eq!(state.connection, ConnectionState::Disconnected);
    state.apply_event(WorkerEvent::Spectrum(spectrum_view()), false);
    assert!(state.spectrum.is_none());
    assert_eq!(state.spectrum_sequence, 0);
}

#[test]
fn disconnect_clears_session_and_reports_reconnect_hint_after_scan() {
    let (mut state, _dir) = isolated_state();
    state.apply_event(WorkerEvent::Connected(device_info()), true);
    state.apply_event(WorkerEvent::Spectrum(spectrum_view()), true);
    state.apply_event(WorkerEvent::Disconnected, true);
    assert_eq!(state.connection, ConnectionState::Disconnected);
    assert!(state.device_info.is_none());
    assert!(state.spectrum.is_none());
    assert_eq!(state.spectrum_sequence, 0);
    assert_eq!(state.status, "Disconnected.");
    state.scanned_once = true;
    state.clear_session();
    assert_eq!(state.status, "Disconnected. Choose a device to reconnect.");
}

#[test]
fn spectrum_event_advances_sequence_and_clears_pending() {
    let (mut state, _dir) = isolated_state();
    state.spectrum_fetch_pending = true;
    state.apply_event(WorkerEvent::Spectrum(spectrum_view()), true);
    assert_eq!(state.spectrum_sequence, 1);
    assert!(!state.spectrum_fetch_pending);
    assert!(state.last_fetch.is_some());
    assert_eq!(state.status, "Live spectrum");
}

#[test]
fn device_status_merges_into_device_info() {
    let (mut state, _dir) = isolated_state();
    state.apply_event(WorkerEvent::Connected(device_info()), true);
    state.monitor_fetch_pending = true;
    state.apply_event(
        WorkerEvent::DeviceStatus(DeviceStatus {
            battery_percent: None,
            temperature_c: Some(27.5),
            rssi_dbm: None,
        }),
        true,
    );
    let info = state.device_info.as_ref().expect("info");
    assert_eq!(info.battery_percent, Some(90.0));
    assert_eq!(info.temperature_c, Some(27.5));
    assert!(!state.monitor_fetch_pending);
}

#[test]
fn monitor_sample_feeds_monitor_and_dosimeter() {
    let (mut state, _dir) = isolated_state();
    state.apply_event(WorkerEvent::Connected(device_info()), true);
    state.monitor_fetch_pending = true;
    state.apply_event(WorkerEvent::MonitorSample(monitor_sample(12.0)), true);
    assert_eq!(state.monitor.history.len(), 1);
    assert_eq!(state.dosimeter.latest.map(|d| d.dose), Some(12.5));
    assert!(state.last_monitor_fetch.is_some());
    assert!(!state.monitor_fetch_pending);
    assert_eq!(state.status, "Live monitor");
}
