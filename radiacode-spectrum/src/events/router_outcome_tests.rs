use super::router_fixture::Fixture;
use super::test_fixtures::{device_info, monitor_sample};
use crate::monitor::AlarmLevel;
use crate::settings::SettingsDeviceOp;
use crate::view_tab::ViewTab;
use crate::worker::WorkerEvent;
use crate::worker::test_fixtures::{alarm_limits, device_config};

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
