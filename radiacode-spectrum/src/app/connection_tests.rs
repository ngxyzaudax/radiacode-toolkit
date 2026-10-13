use super::test_harness::{AppHarness, bluetooth_endpoint, discovered_bluetooth, has_command};
use crate::device::DeviceAction;
use crate::model::ConnectionState;
use crate::ui_toolbar::SpectrumToolbarAction;
use crate::view_tab::ViewTab;
use crate::worker::{WorkerCommand, WorkerEvent};

#[test]
fn scan_action_marks_scanning_and_sends_scan_command() {
    let mut harness = AppHarness::new();
    harness.app.handle_device_action(DeviceAction::Scan);
    assert!(harness.app.state.scanning);
    assert!(has_command(&harness.drain_commands(), |command| matches!(
        command,
        WorkerCommand::Scan
    )));
}

#[test]
fn connect_uses_advertised_rssi_as_hint() {
    let mut harness = AppHarness::new();
    harness.app.state.devices = vec![discovered_bluetooth(Some(-70))];
    harness
        .app
        .handle_device_action(DeviceAction::Connect(bluetooth_endpoint()));
    assert_eq!(harness.app.state.connection, ConnectionState::Connecting);
    assert_eq!(
        harness.app.state.connecting_endpoint,
        Some(bluetooth_endpoint())
    );
    let commands = harness.drain_commands();
    assert!(has_command(&commands, |command| matches!(
        command,
        WorkerCommand::Connect {
            hint_rssi: Some(-70),
            ..
        }
    )));
}

#[test]
fn connected_event_remembers_device_and_switches_to_monitor() {
    let mut harness = AppHarness::new();
    harness.app.start_connect(bluetooth_endpoint());
    harness.deliver(WorkerEvent::Connected(super::test_harness::connected_info()));
    assert_eq!(harness.app.state.connection, ConnectionState::Connected);
    assert_eq!(harness.app.tabs.active, ViewTab::Monitor);
    assert_eq!(
        harness.app.settings.app.last_endpoint,
        Some(bluetooth_endpoint())
    );
    let commands = harness.drain_commands();
    assert!(has_command(&commands, |command| matches!(
        command,
        WorkerCommand::SetCaptureInterval(_)
    )));
    assert!(has_command(&commands, |command| matches!(
        command,
        WorkerCommand::SetMonitorPollInterval(_)
    )));
    assert!(has_command(&commands, |command| matches!(
        command,
        WorkerCommand::FetchSpectrum
    )));
}

#[test]
fn disconnect_ends_session_and_blocks_stale_events() {
    let mut harness = AppHarness::connected();
    let generation = harness.app.worker.session_generation();
    harness.app.handle_device_action(DeviceAction::Disconnect);
    assert_eq!(harness.app.worker.session_generation(), generation + 1);
    assert!(harness.app.session_blocked);
    assert_eq!(harness.app.state.connection, ConnectionState::Disconnected);
    assert!(has_command(&harness.drain_commands(), |command| matches!(
        command,
        WorkerCommand::Disconnect
    )));
    harness.deliver(WorkerEvent::Disconnected);
    assert!(!harness.app.session_blocked);
}

#[test]
fn scan_finished_auto_connects_to_remembered_device() {
    let mut harness = AppHarness::new();
    harness.app.settings.app.auto_connect = true;
    harness.app.settings.app.last_endpoint = Some(bluetooth_endpoint());
    harness.deliver(WorkerEvent::ScanFinished(vec![discovered_bluetooth(None)]));
    assert!(harness.app.auto_connect_attempted);
    assert_eq!(harness.app.state.connection, ConnectionState::Connecting);
    harness.deliver(WorkerEvent::Disconnected);
    harness.deliver(WorkerEvent::ScanFinished(vec![discovered_bluetooth(None)]));
    assert_eq!(harness.app.state.connection, ConnectionState::Disconnected);
}

#[test]
fn auto_connect_requires_discovered_remembered_device() {
    let mut harness = AppHarness::new();
    harness.app.settings.app.auto_connect = true;
    harness.app.settings.app.last_endpoint = Some(bluetooth_endpoint());
    harness.deliver(WorkerEvent::ScanFinished(Vec::new()));
    assert!(!harness.app.auto_connect_attempted);
    assert_eq!(harness.app.state.connection, ConnectionState::Disconnected);
}

#[test]
fn spectrum_reset_marks_fetch_pending() {
    let mut harness = AppHarness::connected();
    harness
        .app
        .handle_spectrum_toolbar_action(SpectrumToolbarAction::Reset);
    assert!(harness.app.state.spectrum_fetch_pending);
    assert!(has_command(&harness.drain_commands(), |command| matches!(
        command,
        WorkerCommand::ResetSpectrum
    )));
}

#[test]
fn stopped_worker_is_reported_in_status() {
    let mut harness = AppHarness::new();
    drop(std::mem::replace(
        &mut harness.commands,
        tokio::sync::mpsc::unbounded_channel().1,
    ));
    harness.app.start_scan();
    assert_eq!(harness.app.state.status, "Device worker stopped.");
}
