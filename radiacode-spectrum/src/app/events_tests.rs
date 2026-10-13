use super::test_harness::{AppHarness, edited, has_command, loaded_harness};
use crate::monitor::MonitorLeaveChoice;
use crate::peak_overlay::SpectrumPlotAction;
use crate::settings::{SettingsAction, SettingsDeviceOp};
use crate::view_tab::ViewTab;
use crate::worker::test_fixtures::device_config;
use crate::worker::{WorkerCommand, WorkerEvent};

#[test]
fn entering_settings_tab_loads_device_config_when_connected() {
    let mut harness = AppHarness::connected();
    harness.app.tabs.active = ViewTab::Settings;
    harness.app.on_tab_changed(ViewTab::Monitor);
    assert_eq!(harness.app.settings.device_op, SettingsDeviceOp::Loading);
    assert!(has_command(&harness.drain_commands(), |command| matches!(
        command,
        WorkerCommand::FetchDeviceConfig
    )));
    harness.deliver(WorkerEvent::DeviceConfig(device_config()));
    assert_eq!(harness.app.settings.draft, Some(device_config()));
}

#[test]
fn entering_settings_tab_with_dirty_draft_asks_for_confirmation() {
    let mut harness = loaded_harness();
    harness.app.settings.draft = Some(edited(device_config()));
    harness.app.tabs.active = ViewTab::Device;
    harness.app.tabs.active = ViewTab::Settings;
    harness.app.on_tab_changed(ViewTab::Device);
    assert!(harness.app.settings.show_load_confirm);
    assert!(harness.drain_commands().is_empty());
}

#[test]
fn leaving_monitor_with_dirty_draft_and_saving_switches_after_save() {
    let mut harness = loaded_harness();
    harness.app.tabs.active = ViewTab::Monitor;
    harness.app.settings.draft = Some(edited(device_config()));
    assert!(!harness.app.try_switch_tab(ViewTab::Spectrum));
    assert!(harness.app.tabs.monitor_leave_open);
    harness
        .app
        .handle_monitor_leave_choice(MonitorLeaveChoice::Save);
    assert_eq!(harness.app.settings.device_op, SettingsDeviceOp::Saving);
    assert!(has_command(&harness.drain_commands(), |command| matches!(
        command,
        WorkerCommand::ApplyDeviceConfig(_)
    )));
    harness.deliver(WorkerEvent::DeviceConfig(edited(device_config())));
    assert_eq!(harness.app.tabs.active, ViewTab::Spectrum);
    assert_eq!(harness.app.settings.status, "Saved to device");
}

#[test]
fn discarding_monitor_changes_switches_immediately() {
    let mut harness = loaded_harness();
    harness.app.tabs.active = ViewTab::Monitor;
    harness.app.settings.draft = Some(edited(device_config()));
    assert!(!harness.app.try_switch_tab(ViewTab::About));
    harness
        .app
        .handle_monitor_leave_choice(MonitorLeaveChoice::Discard);
    assert_eq!(harness.app.tabs.active, ViewTab::About);
    assert!(!harness.app.settings.draft_dirty());
    harness.app.tabs.pending = Some(ViewTab::Spectrum);
    harness
        .app
        .handle_monitor_leave_choice(MonitorLeaveChoice::Stay);
    assert!(harness.app.tabs.pending.is_none());
}

#[test]
fn failed_save_reopens_leave_dialog() {
    let mut harness = loaded_harness();
    harness.app.tabs.active = ViewTab::Monitor;
    harness.app.settings.draft = Some(edited(device_config()));
    harness.app.try_switch_tab(ViewTab::Spectrum);
    harness
        .app
        .handle_monitor_leave_choice(MonitorLeaveChoice::Save);
    harness.deliver(WorkerEvent::Error("write failed".into()));
    assert_eq!(harness.app.settings.status, "write failed");
    assert!(harness.app.tabs.monitor_leave_open);
    assert_eq!(harness.app.tabs.pending, Some(ViewTab::Spectrum));
}

#[test]
fn clock_sync_and_app_settings_actions_send_commands() {
    let mut harness = AppHarness::connected();
    harness
        .app
        .handle_settings_action(SettingsAction::SyncClock);
    harness
        .app
        .handle_settings_action(SettingsAction::AppChanged);
    let commands = harness.drain_commands();
    assert!(has_command(&commands, |command| matches!(
        command,
        WorkerCommand::SyncDeviceClock
    )));
    assert!(has_command(&commands, |command| matches!(
        command,
        WorkerCommand::SetMonitorPollInterval(_)
    )));
}

#[test]
fn spectrogram_settings_change_syncs_capture_interval_when_changed() {
    let mut harness = AppHarness::connected();
    harness
        .app
        .handle_settings_action(SettingsAction::SpectrogramChanged);
    assert!(harness.drain_commands().is_empty());
    harness.app.settings.spectrogram.capture_interval_secs += 10.0;
    harness
        .app
        .handle_settings_action(SettingsAction::SpectrogramChanged);
    assert!(has_command(&harness.drain_commands(), |command| matches!(
        command,
        WorkerCommand::SetCaptureInterval(_)
    )));
}

#[test]
fn plot_actions_open_catalogue_tab() {
    let mut harness = AppHarness::new();
    let u238 = radiacode_nuclides::NuclideId::new(92, 146, 0);
    harness
        .app
        .handle_spectrum_plot_action(SpectrumPlotAction::OpenCatalogue(u238));
    assert_eq!(harness.app.tabs.active, ViewTab::Catalogue);
    assert_eq!(harness.app.catalogue.selected, Some(u238));
}
