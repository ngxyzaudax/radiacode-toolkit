use tracing::info;

use crate::energy::energy_grid;
use crate::model::SpectrumView;
use crate::spectrogram::capture::SpectrogramCapture;
use crate::spectrogram::capture_lock::{lock_capture, lock_progress};
use crate::spectrogram::recording_seed::{
    ensure_live_series, recording_header, seed_writer_from_live,
};
use crate::spectrogram::state::SpectrogramState;
use crate::spectrogram::storage::{
    RecordingWriter, ensure_dir, open_recording_append, timestamp_filename,
};

pub fn start_recording(
    state: &mut SpectrogramState,
    spectrum: Option<&SpectrumView>,
    device_serial: Option<&str>,
) -> Result<(), String> {
    let mut cap = lock_capture(&state.capture);
    if cap.recording.is_some() {
        return Ok(());
    }
    let spectrum = spectrum.ok_or("Connect a device before recording.")?;
    let grid = energy_grid(spectrum);
    if grid.indices.is_empty() {
        return Err("No channels in energy range.".into());
    }
    ensure_live_series(&mut cap, spectrum, device_serial, &grid.energies_kev);
    let mut writer = create_writer(&cap, spectrum, device_serial, grid.indices.len() as u32)?;
    let live_series = lock_progress(&cap.progress).live_series.clone();
    let seeded_rows = seed_writer_from_live(&mut writer, live_series.as_deref())
        .map_err(|error| error.to_string())?;
    let mut progress = lock_progress(&cap.progress);
    let continue_live = seeded_rows > 0 && progress.baseline.is_some();
    progress.begin_capture(!continue_live);
    progress.paused_recording_path = None;
    drop(progress);
    cap.recording = Some(writer);
    Ok(())
}

fn create_writer(
    cap: &SpectrogramCapture,
    spectrum: &SpectrumView,
    device_serial: Option<&str>,
    channel_count: u32,
) -> Result<RecordingWriter, String> {
    let dir = ensure_dir(&cap.settings.recordings_dir).map_err(|error| error.to_string())?;
    let path = dir.join(timestamp_filename());
    let header = recording_header(cap, spectrum, device_serial, channel_count);
    info!(path = %path.display(), "spectrogram recording started");
    RecordingWriter::create(path, &header).map_err(|error| error.to_string())
}

pub fn resume_recording(
    state: &mut SpectrogramState,
    spectrum: Option<&SpectrumView>,
    device_serial: Option<&str>,
) -> Result<(), String> {
    let mut cap = lock_capture(&state.capture);
    if cap.recording.is_some() {
        return Ok(());
    }
    let Some(path) = lock_progress(&cap.progress).paused_recording_path.clone() else {
        drop(cap);
        return start_recording(state, spectrum, device_serial);
    };
    let spectrum = spectrum.ok_or("Connect a device before resuming.")?;
    let grid = energy_grid(spectrum);
    ensure_live_series(&mut cap, spectrum, device_serial, &grid.energies_kev);
    let writer = open_recording_append(path.clone()).map_err(|error| error.to_string())?;
    cap.recording = Some(writer);
    lock_progress(&cap.progress).begin_capture(true);
    info!(path = %path.display(), "spectrogram recording resumed");
    Ok(())
}
