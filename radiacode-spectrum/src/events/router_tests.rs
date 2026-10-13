use std::sync::{Arc, Mutex};

use super::test_fixtures::{device_info, isolated_data_dir, monitor_sample};
use super::{AppState, EventOutcome, EventRouter};
use crate::model::ConnectionState;
use crate::monitor::AlarmLevel;
use crate::settings::{SettingsDeviceOp, SettingsState};
use crate::spectrogram::SpectrogramState;
use crate::spectrogram::capture::SpectrogramCapture;
use crate::usb_access::UsbAccessPrompt;
use crate::view_tab::ViewTab;
use crate::worker::test_fixtures::{alarm_limits, device_config};
use crate::worker::{WorkerCommand, WorkerEvent};

struct Fixture {
    state: AppState,
    settings: SettingsState,
    spectrogram: SpectrogramState,
    usb_prompt: Option<UsbAccessPrompt>,
    pending_after_save: Option<ViewTab>,
    pending: Option<ViewTab>,
    monitor_leave_open: bool,
    last_alarm_level: AlarmLevel,
    active_tab: ViewTab,
    session_blocked: bool,
    _dir: tempfile::TempDir,
}

impl Fixture {
    fn new(active_tab: ViewTab) -> Self {
        let dir = isolated_data_dir();
        let capture = Arc::new(Mutex::new(SpectrogramCapture::new()));
        Self {
            state: AppState::new(),
            settings: SettingsState::new(),
            spectrogram: SpectrogramState::new(capture),
            usb_prompt: None,
            pending_after_save: None,
            pending: None,
            monitor_leave_open: false,
            last_alarm_level: AlarmLevel::Normal,
            active_tab,
            session_blocked: false,
            _dir: dir,
        }
    }

    fn dispatch(&mut self, event: WorkerEvent) -> (EventOutcome, Vec<WorkerCommand>) {
        let mut sent = Vec::new();
        let mut router = EventRouter {
            state: &mut self.state,
            settings: &mut self.settings,
            spectrogram: &mut self.spectrogram,
            usb_prompt: &mut self.usb_prompt,
            session_blocked: self.session_blocked,
            active_tab: self.active_tab,
            pending_tab_after_save: &mut self.pending_after_save,
            pending_tab: &mut self.pending,
            monitor_leave_open: &mut self.monitor_leave_open,
            last_alarm_level: &mut self.last_alarm_level,
            pc_alarm_repeat: false,
        };
        let outcome = router.dispatch(event, &mut |command| sent.push(command));
        (outcome, sent)
    }
}

#[test]
fn initial_connect_syncs_intervals_and_switches_to_monitor() {
    let mut fixture = Fixture::new(ViewTab::Device);
    fixture.state.connection = ConnectionState::Connecting;
    let (outcome, sent) = fixture.dispatch(WorkerEvent::Connected(device_info()));
    assert!(outcome.fetch_spectrum);
    assert!(outcome.sync_capture_interval);
    assert!(outcome.sync_monitor_poll_interval);
    assert_eq!(outcome.switch_tab, Some(ViewTab::Monitor));
    assert!(outcome.remember_device.is_some());
    assert_eq!(sent.len(), 1);
    assert!(matches!(sent[0], WorkerCommand::FetchMonitor));
    assert!(fixture.state.monitor_fetch_pending);
}

#[test]
fn initial_connect_on_config_tab_requests_device_config() {
    let mut fixture = Fixture::new(ViewTab::Settings);
    fixture.state.connection = ConnectionState::Connecting;
    let (outcome, sent) = fixture.dispatch(WorkerEvent::Connected(device_info()));
    assert!(outcome.switch_tab.is_none());
    assert!(
        sent.iter()
            .any(|command| matches!(command, WorkerCommand::FetchDeviceConfig))
    );
}

#[test]
fn reconnect_does_not_resync_or_remember() {
    let mut fixture = Fixture::new(ViewTab::Spectrum);
    fixture.state.connection = ConnectionState::Connected;
    let (outcome, _) = fixture.dispatch(WorkerEvent::Connected(device_info()));
    assert!(outcome.fetch_spectrum);
    assert!(!outcome.sync_capture_interval);
    assert!(outcome.remember_device.is_none());
}

#[test]
fn blocked_session_ignores_connect() {
    let mut fixture = Fixture::new(ViewTab::Device);
    fixture.session_blocked = true;
    let (outcome, sent) = fixture.dispatch(WorkerEvent::Connected(device_info()));
    assert!(!outcome.fetch_spectrum);
    assert!(sent.is_empty());
    assert_eq!(fixture.state.connection, ConnectionState::Disconnected);
}

#[test]
fn device_config_completes_pending_load_and_save() {
    let mut fixture = Fixture::new(ViewTab::Settings);
    fixture.settings.begin_load();
    fixture.dispatch(WorkerEvent::DeviceConfig(device_config()));
    assert_eq!(fixture.settings.device_op, SettingsDeviceOp::Idle);
    assert!(fixture.settings.draft.is_some());
    assert_eq!(fixture.state.monitor.limits, Some(alarm_limits()));
    fixture.settings.begin_save();
    fixture.pending_after_save = Some(ViewTab::Spectrum);
    let (outcome, _) = fixture.dispatch(WorkerEvent::DeviceConfig(device_config()));
    assert_eq!(outcome.switch_tab, Some(ViewTab::Spectrum));
    assert_eq!(fixture.settings.status, "Saved to device");
}

#[test]
fn error_during_save_reopens_leave_dialog() {
    let mut fixture = Fixture::new(ViewTab::Monitor);
    fixture.settings.begin_save();
    fixture.pending_after_save = Some(ViewTab::Spectrum);
    fixture.dispatch(WorkerEvent::Error("write failed".into()));
    assert_eq!(fixture.settings.device_op, SettingsDeviceOp::Idle);
    assert_eq!(fixture.settings.status, "write failed");
    assert_eq!(fixture.pending, Some(ViewTab::Spectrum));
    assert!(fixture.pending_after_save.is_none());
    assert!(fixture.monitor_leave_open);
}

#[test]
fn error_after_save_replaces_saved_status() {
    let mut fixture = Fixture::new(ViewTab::Settings);
    fixture.settings.status = "Saved to device".into();
    fixture.dispatch(WorkerEvent::Error("clock sync failed".into()));
    assert_eq!(fixture.settings.status, "clock sync failed");
}

#[test]
fn disconnect_resets_settings_and_alarm_level() {
    let mut fixture = Fixture::new(ViewTab::Monitor);
    fixture.last_alarm_level = AlarmLevel::Danger;
    fixture.settings.begin_load();
    fixture.dispatch(WorkerEvent::Disconnected);
    assert_eq!(fixture.last_alarm_level, AlarmLevel::Normal);
    assert_eq!(fixture.settings.device_op, SettingsDeviceOp::Idle);
}

#[test]
fn monitor_sample_tracks_rising_alarm_level() {
    let mut fixture = Fixture::new(ViewTab::Monitor);
    fixture.dispatch(WorkerEvent::Connected(device_info()));
    fixture.dispatch(WorkerEvent::AlarmLimits(alarm_limits()));
    let (outcome, _) = fixture.dispatch(WorkerEvent::MonitorSample(monitor_sample(100.0)));
    assert!(outcome.monitor_sample);
    assert_eq!(fixture.last_alarm_level, AlarmLevel::Danger);
    fixture.dispatch(WorkerEvent::MonitorSample(monitor_sample(1.0)));
    assert_eq!(fixture.last_alarm_level, AlarmLevel::Normal);
}

#[test]
fn scan_finished_is_reported_in_outcome() {
    let mut fixture = Fixture::new(ViewTab::Device);
    let (outcome, _) = fixture.dispatch(WorkerEvent::ScanFinished(Vec::new()));
    assert!(outcome.scan_finished);
    assert!(fixture.state.scanned_once);
}
