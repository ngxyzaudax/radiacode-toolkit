use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::model::SpectrumView;
use crate::persist::json_store::use_test_data_dir;
use crate::spectrogram::capture::SpectrogramCapture;
use crate::spectrogram::capture_lock::{lock_capture, lock_progress};
use crate::spectrogram::model::SpectrogramDisplay;
use crate::spectrogram::state::SpectrogramState;
use crate::spectrogram::test_fixtures::write_fixture_recording;

struct Fixture {
    state: SpectrogramState,
    _data_dir: tempfile::TempDir,
    recordings_dir: tempfile::TempDir,
}

fn fixture() -> Fixture {
    let data_dir = tempfile::tempdir().unwrap();
    use_test_data_dir(data_dir.path().to_path_buf());
    let recordings_dir = tempfile::tempdir().unwrap();
    let capture = Arc::new(Mutex::new(SpectrogramCapture::new()));
    let mut state = SpectrogramState::new(Arc::clone(&capture));
    state.settings.recordings_dir = recordings_dir.path().display().to_string();
    state.settings.capture_interval_secs = 10.0;
    state.on_settings_changed();
    lock_capture(&capture).on_session_connect("SN1");
    state.sync_from_capture();
    Fixture {
        state,
        _data_dir: data_dir,
        recordings_dir,
    }
}

fn spectrum(total: u32, duration_secs: u64) -> SpectrumView {
    SpectrumView {
        duration: Duration::from_secs(duration_secs),
        a0: 0.0,
        a1: 1.0,
        a2: 0.0,
        counts: vec![total; 64],
        total_counts: u64::from(total) * 64,
    }
}

fn is_capture_enabled(state: &SpectrogramState) -> bool {
    let cap = lock_capture(&state.capture);
    let progress = lock_progress(&cap.progress);
    progress.capture_enabled
}

#[test]
fn recording_requires_a_spectrum() {
    let mut fx = fixture();
    let error = fx.state.start_recording(None, None).unwrap_err();
    assert_eq!(error, "Connect a device before recording.");
    assert!(!fx.state.is_recording());
}

#[test]
fn start_stop_cycle_writes_file_and_opens_editor() {
    let mut fx = fixture();
    fx.state
        .start_recording(Some(&spectrum(5, 5)), Some("SN1"))
        .unwrap();
    assert!(fx.state.is_recording());
    fx.state
        .start_recording(Some(&spectrum(5, 5)), Some("SN1"))
        .unwrap();
    fx.state.ingest_spectrum(&spectrum(5, 5), None, 1);
    fx.state.ingest_spectrum(&spectrum(15, 15), None, 2);
    fx.state.stop_recording().unwrap();
    assert!(!fx.state.is_recording());
    assert!(fx.state.can_resume_append());
    assert_eq!(fx.state.history.len(), 1);
    assert_eq!(fx.state.history[0].row_count, 1);
    assert_eq!(fx.state.library_edit_path, fx.state.paused_recording_path);
    assert_eq!(fx.state.library_edit_name, fx.state.history[0].name);
}

#[test]
fn resume_appends_to_paused_recording() {
    let mut fx = fixture();
    fx.state
        .start_recording(Some(&spectrum(5, 5)), Some("SN1"))
        .unwrap();
    fx.state.ingest_spectrum(&spectrum(5, 5), None, 1);
    fx.state.ingest_spectrum(&spectrum(15, 15), None, 2);
    fx.state.stop_recording().unwrap();
    fx.state
        .resume_recording(Some(&spectrum(15, 15)), Some("SN1"))
        .unwrap();
    assert!(fx.state.is_recording());
    fx.state.ingest_spectrum(&spectrum(25, 25), None, 3);
    fx.state.ingest_spectrum(&spectrum(35, 35), None, 4);
    fx.state.stop_recording().unwrap();
    assert_eq!(fx.state.history.len(), 1);
    assert_eq!(fx.state.history[0].row_count, 2);
}

#[test]
fn resume_without_paused_recording_starts_fresh() {
    let mut fx = fixture();
    fx.state
        .resume_recording(Some(&spectrum(5, 5)), Some("SN1"))
        .unwrap();
    assert!(fx.state.is_recording());
    assert!(fx.state.resume_recording(None, None).is_ok());
}

#[test]
fn pause_and_resume_toggle_capture_flag() {
    let mut fx = fixture();
    assert!(is_capture_enabled(&fx.state));
    fx.state.pause_capture().unwrap();
    assert!(!is_capture_enabled(&fx.state));
    assert!(fx.state.is_capture_paused());
    fx.state.resume_capture().unwrap();
    assert!(is_capture_enabled(&fx.state));
}

#[test]
fn request_load_switches_display_and_reports_missing_files() {
    let mut fx = fixture();
    let path = write_fixture_recording(fx.recordings_dir.path(), "lib", &[vec![1, 2, 3]]);
    fx.state.request_load(path.clone());
    assert_eq!(fx.state.display, SpectrogramDisplay::Loaded);
    assert_eq!(fx.state.loaded_path.as_deref(), Some(path.as_path()));
    assert!(fx.state.error.is_empty());
    fx.state.close_loaded();
    assert_eq!(fx.state.display, SpectrogramDisplay::Live);
    fx.state
        .request_load(fx.recordings_dir.path().join("missing.rcwf"));
    assert!(fx.state.error.starts_with("Load failed"));
}

#[test]
fn settings_change_propagates_to_capture_and_refreshes_history() {
    let mut fx = fixture();
    let other_dir = tempfile::tempdir().unwrap();
    write_fixture_recording(other_dir.path(), "elsewhere", &[vec![1, 2, 3]]);
    fx.state.settings.recordings_dir = other_dir.path().display().to_string();
    fx.state.settings.capture_interval_secs = 20.0;
    fx.state.on_settings_changed();
    let cap = lock_capture(&fx.state.capture);
    assert_eq!(cap.settings.capture_interval_secs, 20.0);
    assert_eq!(
        cap.settings.recordings_dir,
        fx.state.settings.recordings_dir
    );
    drop(cap);
    assert_eq!(fx.state.history.len(), 1);
    assert_eq!(fx.state.history[0].name, "elsewhere");
}
