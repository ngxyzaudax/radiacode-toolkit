use std::path::PathBuf;

use tracing::info;

use crate::energy::energy_grid;
use crate::model::SpectrumView;
use crate::spectrogram::model::SpectrogramDisplay;
use crate::spectrogram::recording_seed::{
    ensure_live_series, recording_header, seed_writer_from_live,
};
use crate::spectrogram::state::SpectrogramState;
use crate::spectrogram::storage::{
    RecordingWriter, ensure_dir, load_recording, open_recording_append, timestamp_filename,
};

pub fn start_recording(
    state: &mut SpectrogramState,
    spectrum: Option<&SpectrumView>,
    device_serial: Option<&str>,
) -> Result<(), String> {
    let mut cap = state
        .capture
        .lock()
        .map_err(|_| "capture lock failed".to_string())?;
    if cap.recording.is_some() {
        return Ok(());
    }
    let Some(spectrum) = spectrum else {
        return Err("Connect a device before recording.".into());
    };
    let grid = energy_grid(spectrum);
    if grid.indices.is_empty() {
        return Err("No channels in energy range.".into());
    }
    ensure_live_series(&mut cap, spectrum, device_serial, &grid.energies_kev);
    let dir = ensure_dir(&cap.settings.recordings_dir).map_err(|error| error.to_string())?;
    let path = dir.join(timestamp_filename());
    let header = recording_header(&cap, spectrum, device_serial, grid.indices.len() as u32);
    info!(path = %path.display(), "spectrogram recording started");
    let mut writer = RecordingWriter::create(path, &header).map_err(|error| error.to_string())?;
    let live_series = cap
        .progress
        .lock()
        .ok()
        .and_then(|progress| progress.live_series.clone());
    let seeded_rows = seed_writer_from_live(&mut writer, live_series.as_deref())
        .map_err(|error| error.to_string())?;
    {
        let mut progress = cap
            .progress
            .lock()
            .map_err(|_| "capture progress lock failed".to_string())?;
        let continue_live = seeded_rows > 0 && progress.baseline.is_some();
        if !continue_live {
            progress.skip_next_sample = true;
        }
        progress.paused_recording_path = None;
        progress.capture_enabled = true;
        progress.last_auto_save = None;
        progress.error.clear();
        progress.mark_dirty();
    }
    cap.recording = Some(writer);
    Ok(())
}

pub fn pause_capture(state: &mut SpectrogramState) -> Result<(), String> {
    let cap = state
        .capture
        .lock()
        .map_err(|_| "capture lock failed".to_string())?;
    let mut progress = cap
        .progress
        .lock()
        .map_err(|_| "capture progress lock failed".to_string())?;
    progress.capture_enabled = false;
    progress.error.clear();
    progress.mark_dirty();
    info!("spectrogram recording paused");
    Ok(())
}

pub fn resume_capture(state: &mut SpectrogramState) -> Result<(), String> {
    let cap = state
        .capture
        .lock()
        .map_err(|_| "capture lock failed".to_string())?;
    let mut progress = cap
        .progress
        .lock()
        .map_err(|_| "capture progress lock failed".to_string())?;
    progress.capture_enabled = true;
    progress.skip_next_sample = true;
    progress.error.clear();
    progress.mark_dirty();
    info!("spectrogram capture resumed");
    Ok(())
}

pub fn stop_recording(state: &mut SpectrogramState) -> Result<(), String> {
    let mut cap = state
        .capture
        .lock()
        .map_err(|_| "capture lock failed".to_string())?;
    let Some(writer) = cap.recording.take() else {
        return Ok(());
    };
    let path = writer.finalize().map_err(|error| error.to_string())?;
    info!(path = %path.display(), "spectrogram recording saved");
    {
        let mut progress = cap
            .progress
            .lock()
            .map_err(|_| "capture progress lock failed".to_string())?;
        progress.paused_recording_path = Some(path.clone());
        progress.error.clear();
        progress.mark_dirty();
    }
    drop(cap);
    state.refresh_history();
    Ok(())
}

pub fn resume_recording(
    state: &mut SpectrogramState,
    spectrum: Option<&SpectrumView>,
    device_serial: Option<&str>,
) -> Result<(), String> {
    let mut cap = state
        .capture
        .lock()
        .map_err(|_| "capture lock failed".to_string())?;
    if cap.recording.is_some() {
        return Ok(());
    }
    let resume_path = cap
        .progress
        .lock()
        .ok()
        .and_then(|progress| progress.paused_recording_path.clone());
    let Some(path) = resume_path else {
        drop(cap);
        return start_recording(state, spectrum, device_serial);
    };
    let Some(spectrum) = spectrum else {
        return Err("Connect a device before resuming.".into());
    };
    let grid = energy_grid(spectrum);
    ensure_live_series(&mut cap, spectrum, device_serial, &grid.energies_kev);
    let writer = open_recording_append(path.clone()).map_err(|error| error.to_string())?;
    cap.recording = Some(writer);
    {
        let mut progress = cap
            .progress
            .lock()
            .map_err(|_| "capture progress lock failed".to_string())?;
        progress.skip_next_sample = true;
        progress.capture_enabled = true;
        progress.last_auto_save = None;
        progress.error.clear();
        progress.mark_dirty();
    }
    info!(path = %path.display(), "spectrogram recording resumed");
    Ok(())
}

pub fn request_load(state: &mut SpectrogramState, path: PathBuf) {
    load_into_state(state, path);
}

pub fn load_into_state(state: &mut SpectrogramState, path: PathBuf) {
    match load_recording(&path) {
        Ok(series) => {
            state.loaded_series = Some(series);
            state.loaded_path = Some(path.clone());
            state.display = SpectrogramDisplay::Loaded;
            if let Some(loaded) = state.loaded_series.as_ref() {
                state.view_range.fit_series_energy(&loaded.energies_kev);
            }
            state.error.clear();
            state.texture.dirty = true;
            state.z_range_rows = 0;
        }
        Err(error) => state.error = format!("Load failed: {error}"),
    }
}
