use std::path::PathBuf;

use tracing::info;

use crate::spectrogram::capture_lock::{lock_capture, lock_progress};
use crate::spectrogram::model::SpectrogramDisplay;
use crate::spectrogram::state::SpectrogramState;
use crate::spectrogram::storage::load_recording;

pub fn pause_capture(state: &mut SpectrogramState) {
    let cap = lock_capture(&state.capture);
    lock_progress(&cap.progress).pause_capture();
    info!("spectrogram recording paused");
}

pub fn resume_capture(state: &mut SpectrogramState) {
    let cap = lock_capture(&state.capture);
    lock_progress(&cap.progress).begin_capture(true);
    info!("spectrogram capture resumed");
}

pub fn stop_recording(state: &mut SpectrogramState) -> Result<(), String> {
    let mut cap = lock_capture(&state.capture);
    let Some(writer) = cap.recording.take() else {
        return Ok(());
    };
    let path = writer.finalize().map_err(|error| error.to_string())?;
    info!(path = %path.display(), "spectrogram recording saved");
    let mut progress = lock_progress(&cap.progress);
    progress.paused_recording_path = Some(path);
    progress.error.clear();
    progress.mark_dirty();
    drop(progress);
    drop(cap);
    state.refresh_history();
    Ok(())
}

pub fn load_into_state(state: &mut SpectrogramState, path: PathBuf) {
    match load_recording(&path) {
        Ok(series) => {
            state.view_range.fit_series_energy(&series.energies_kev);
            state.loaded_series = Some(series);
            state.loaded_path = Some(path);
            state.display = SpectrogramDisplay::Loaded;
            state.error.clear();
            state.texture.dirty = true;
            state.z_range_rows = 0;
        }
        Err(error) => state.error = format!("Load failed: {error}"),
    }
}
