use radiacode_lib::config::DeviceConfig;

use crate::app_config::load_app_config;
use crate::persist::json_store::use_test_data_dir;
use crate::settings::state::{SettingsDeviceOp, SettingsState};
use crate::spectrogram::settings::{SpectrogramSettings, load_settings};
use crate::worker::test_fixtures::device_config;

fn isolated_state() -> (SettingsState, tempfile::TempDir) {
    let data_dir = tempfile::tempdir().expect("data dir");
    use_test_data_dir(data_dir.path().to_path_buf());
    (SettingsState::new(), data_dir)
}

fn edited(config: DeviceConfig) -> DeviceConfig {
    DeviceConfig {
        brightness: config.brightness + 1,
        ..config
    }
}

#[test]
fn fresh_state_needs_auto_load_and_is_idle() {
    let (state, _dir) = isolated_state();
    assert!(state.needs_auto_load());
    assert!(!state.device_busy());
    assert!(!state.draft_dirty());
}

#[test]
fn load_cycle_updates_status_and_baseline() {
    let (mut state, _dir) = isolated_state();
    state.begin_load();
    assert_eq!(state.device_op, SettingsDeviceOp::Loading);
    assert!(state.device_busy());
    assert!(!state.needs_auto_load());
    state.on_loaded(device_config());
    assert_eq!(state.baseline, Some(device_config()));
    assert_eq!(state.draft, Some(device_config()));
    assert!(state.status.is_empty());
    assert!(!state.needs_auto_load());
}

#[test]
fn dirty_draft_requires_confirmation_before_reload() {
    let (mut state, _dir) = isolated_state();
    state.on_loaded(device_config());
    state.request_load();
    assert!(!state.show_load_confirm);
    state.draft = Some(edited(device_config()));
    assert!(state.draft_dirty());
    state.request_load();
    assert!(state.show_load_confirm);
    state.discard();
    assert!(!state.show_load_confirm);
    assert!(!state.draft_dirty());
}

#[test]
fn save_cycle_reports_outcome() {
    let (mut state, _dir) = isolated_state();
    state.on_loaded(device_config());
    state.begin_save();
    assert_eq!(state.device_op, SettingsDeviceOp::Saving);
    state.on_saved(edited(device_config()));
    assert_eq!(state.status, "Saved to device");
    assert_eq!(state.baseline, Some(edited(device_config())));
    state.begin_save();
    state.on_device_op_failed("write failed".into());
    assert_eq!(state.device_op, SettingsDeviceOp::Idle);
    assert_eq!(state.status, "write failed");
}

#[test]
fn disconnect_clears_device_state() {
    let (mut state, _dir) = isolated_state();
    state.on_loaded(device_config());
    state.show_load_confirm = true;
    state.on_disconnect();
    assert!(state.baseline.is_none());
    assert!(state.draft.is_none());
    assert!(!state.show_load_confirm);
    assert!(state.needs_auto_load());
}

#[test]
fn persisting_app_settings_clamps_and_writes() {
    let (mut state, _dir) = isolated_state();
    state.app.monitor_poll_secs = 500;
    state.persist_app();
    assert_eq!(state.app.monitor_poll_secs, 60);
    assert_eq!(load_app_config().monitor_poll_secs, 60);
    assert!(state.status.is_empty());
}

#[test]
fn persisting_spectrogram_settings_clamps_and_applies() {
    let (mut state, _dir) = isolated_state();
    state.spectrogram.z_min = 50.0;
    state.spectrogram.z_max = 10.0;
    state.persist_spectrogram();
    assert!(state.spectrogram.z_max > state.spectrogram.z_min);
    assert_eq!(load_settings().z_min, state.spectrogram.z_min);
    let mut target = SpectrogramSettings::default();
    state.apply_spectrogram_to(&mut target);
    assert_eq!(target.z_min, state.spectrogram.z_min);
}
