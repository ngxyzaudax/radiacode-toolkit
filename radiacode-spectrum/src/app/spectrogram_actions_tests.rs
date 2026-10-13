use radiacode_core::DeviceEndpoint;
use radiacode_usb::UsbAccessStatus;

use super::test_harness::{AppHarness, has_command};
use crate::events::test_fixtures::spectrum_view;
use crate::spectrogram::controls_action::SpectrogramControlsAction;
use crate::spectrogram::model::SpectrogramDisplay;
use crate::spectrogram::storage::ensure_dir;
use crate::spectrogram::test_fixtures::write_fixture_recording;
use crate::usb_access::{UsbAccessAction, UsbAccessPrompt};
use crate::worker::{WorkerCommand, WorkerEvent};

fn recordings_dir(harness: &AppHarness) -> std::path::PathBuf {
    harness
        .app
        .spectrogram
        .settings
        .recordings_dir
        .clone()
        .into()
}

#[test]
fn recording_without_spectrum_reports_error() {
    let mut harness = AppHarness::connected();
    harness
        .app
        .handle_spectrogram_action(SpectrogramControlsAction::StartRecording);
    assert_eq!(
        harness.app.spectrogram.error,
        "Connect a device before recording."
    );
    assert!(!harness.app.spectrogram.is_recording());
}

#[test]
fn recording_cycle_updates_history_and_compare_library() {
    let mut harness = AppHarness::connected();
    harness.deliver(WorkerEvent::Spectrum(spectrum_view()));
    harness
        .app
        .handle_spectrogram_action(SpectrogramControlsAction::StartRecording);
    assert!(harness.app.spectrogram.is_recording());
    assert!(harness.app.spectrogram.error.is_empty());
    harness
        .app
        .handle_spectrogram_action(SpectrogramControlsAction::StopRecording);
    assert!(!harness.app.spectrogram.is_recording());
    assert_eq!(harness.app.spectrogram.history.len(), 1);
    harness
        .app
        .handle_spectrogram_action(SpectrogramControlsAction::LibraryChanged);
    assert_eq!(harness.app.compare.library.len(), 1);
    harness
        .app
        .handle_spectrogram_action(SpectrogramControlsAction::ResumeRecording);
    assert!(harness.app.spectrogram.is_recording());
}

#[test]
fn capture_pause_resume_and_reset_clear_errors() {
    let mut harness = AppHarness::connected();
    harness.app.spectrogram.error = "stale".into();
    harness
        .app
        .handle_spectrogram_action(SpectrogramControlsAction::PauseCapture);
    assert!(harness.app.spectrogram.is_capture_paused());
    assert!(harness.app.spectrogram.error.is_empty());
    harness
        .app
        .handle_spectrogram_action(SpectrogramControlsAction::ResumeCapture);
    assert!(!harness.app.spectrogram.is_capture_paused());
    harness
        .app
        .handle_spectrogram_action(SpectrogramControlsAction::ResetAccumulation);
    assert_eq!(harness.app.spectrogram.live_row_count(), 0);
}

#[test]
fn load_and_close_switch_display_mode() {
    let mut harness = AppHarness::new();
    let path = write_fixture_recording(&recordings_dir(&harness), "lib", &[vec![1, 2, 3]]);
    harness
        .app
        .handle_spectrogram_action(SpectrogramControlsAction::Load(path));
    assert_eq!(harness.app.spectrogram.display, SpectrogramDisplay::Loaded);
    harness
        .app
        .handle_spectrogram_action(SpectrogramControlsAction::CloseLoaded);
    assert_eq!(harness.app.spectrogram.display, SpectrogramDisplay::Live);
}

#[test]
fn spectrogram_settings_change_propagates_to_worker_when_interval_changes() {
    let mut harness = AppHarness::new();
    harness
        .app
        .handle_spectrogram_action(SpectrogramControlsAction::SettingsChanged);
    assert!(harness.drain_commands().is_empty());
    harness.app.spectrogram.settings.capture_interval_secs += 5.0;
    harness
        .app
        .handle_spectrogram_action(SpectrogramControlsAction::SettingsChanged);
    assert_eq!(
        harness.app.settings.spectrogram.capture_interval_secs,
        harness.app.spectrogram.settings.capture_interval_secs
    );
    assert!(has_command(&harness.drain_commands(), |command| matches!(
        command,
        WorkerCommand::SetCaptureInterval(_)
    )));
}

#[test]
fn dismissing_usb_prompt_resets_pending_connection() {
    let mut harness = AppHarness::new();
    let endpoint = DeviceEndpoint::Usb {
        serial: "RC-103-000001".into(),
    };
    harness.app.state.connecting_endpoint = Some(endpoint.clone());
    harness.app.state.status = "USB access required.".into();
    harness.app.usb_access_prompt =
        Some(UsbAccessPrompt::new(endpoint, UsbAccessStatus::RuleMissing));
    harness
        .app
        .handle_usb_access_action(UsbAccessAction::Dismiss);
    assert!(harness.app.usb_access_prompt.is_none());
    assert!(harness.app.state.connecting_endpoint.is_none());
    assert_eq!(harness.app.state.status, "USB setup cancelled.");
    harness
        .app
        .handle_usb_access_action(UsbAccessAction::Dismiss);
    assert_eq!(harness.app.state.status, "USB setup cancelled.");
}
