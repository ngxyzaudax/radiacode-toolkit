use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::model::SpectrumView;
use crate::persist::json_store::use_test_data_dir;
use crate::spectrogram::capture::SpectrogramCapture;
use crate::spectrogram::capture_lock::lock_capture;
use crate::spectrogram::model::{RowKind, SpectrogramRow};
use crate::spectrogram::state::SpectrogramState;
use crate::spectrogram::storage::{RecordingWriter, header_now};

pub const FIXTURE_CHANNELS: u32 = 3;
pub const FIXTURE_INTERVAL_SECS: f64 = 10.0;
pub const FIXTURE_SERIAL: &str = "SN1";

pub struct StateFixture {
    pub state: SpectrogramState,
    pub recordings_dir: tempfile::TempDir,
    _data_dir: tempfile::TempDir,
}

pub fn state_fixture() -> StateFixture {
    state_fixture_with_interval(FIXTURE_INTERVAL_SECS)
}

pub fn state_fixture_with_interval(capture_interval_secs: f64) -> StateFixture {
    let data_dir = tempfile::tempdir().expect("data dir");
    use_test_data_dir(data_dir.path().to_path_buf());
    let recordings_dir = tempfile::tempdir().expect("recordings dir");
    let capture = Arc::new(Mutex::new(SpectrogramCapture::new()));
    let mut state = SpectrogramState::new(Arc::clone(&capture));
    state.settings.recordings_dir = dir_string(recordings_dir.path());
    state.settings.capture_interval_secs = capture_interval_secs;
    state.on_settings_changed();
    lock_capture(&capture).on_session_connect(FIXTURE_SERIAL);
    state.sync_from_capture();
    StateFixture {
        state,
        recordings_dir,
        _data_dir: data_dir,
    }
}

pub fn spectrum_view(total: u32, duration_secs: u64, channels: usize) -> SpectrumView {
    SpectrumView {
        duration: Duration::from_secs(duration_secs),
        a0: 0.0,
        a1: 1.0,
        a2: 0.0,
        counts: vec![total; channels],
        total_counts: u64::from(total) * channels as u64,
    }
}

pub fn write_fixture_recording(dir: &Path, file_stem: &str, rows: &[Vec<u32>]) -> PathBuf {
    let path = dir.join(format!("{file_stem}.rcwf"));
    let header = header_now(
        0.0,
        1.0,
        0.0,
        FIXTURE_CHANNELS,
        FIXTURE_INTERVAL_SECS,
        Some("SN-FIXTURE".into()),
        vec![10.0, 20.0, 30.0],
    );
    let mut writer = RecordingWriter::create(path.clone(), &header).expect("create recording");
    for (index, counts) in rows.iter().enumerate() {
        writer
            .append_row(&SpectrogramRow {
                elapsed_secs: index as f64 * FIXTURE_INTERVAL_SECS,
                interval_secs: FIXTURE_INTERVAL_SECS,
                kind: RowKind::Normal,
                counts: counts.clone(),
            })
            .expect("append row");
    }
    writer.finalize().expect("finalize recording")
}

pub fn dir_string(dir: &Path) -> String {
    dir.display().to_string()
}
