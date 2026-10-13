use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::spectrogram::library_meta::{LibraryMeta, meta_path, save_meta};
use crate::spectrogram::model::SpectrogramSeries;
use crate::spectrogram::storage::{
    RecordingWriter, spectrograms_dir, timestamp_filename, write_recording,
};

pub const AUTOSAVE_RETAIN: usize = 3;
const RECORDING_EXTENSION: &str = "rcwf";

pub fn auto_save_snapshot(
    series: &SpectrogramSeries,
    writer: Option<&RecordingWriter>,
    recordings_dir: &str,
) -> std::io::Result<PathBuf> {
    let dir = spectrograms_dir(recordings_dir).join("autosave");
    fs::create_dir_all(&dir)?;
    let path = unique_path(dir.join(format!("autosave_{}", timestamp_filename())));
    write_recording(&path, series)?;
    if let Some(writer) = writer {
        let _ = save_meta(&path, &autosave_meta(&writer.path));
    }
    trim_autosave_dir(&dir, AUTOSAVE_RETAIN)?;
    Ok(path)
}

fn unique_path(candidate: PathBuf) -> PathBuf {
    if !candidate.exists() {
        return candidate;
    }
    let stem = candidate
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("autosave");
    (1..)
        .map(|suffix| candidate.with_file_name(format!("{stem}_{suffix}.{RECORDING_EXTENSION}")))
        .find(|path| !path.exists())
        .unwrap_or(candidate)
}

fn autosave_meta(recording_path: &Path) -> LibraryMeta {
    let fallback = recording_path
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("recording");
    LibraryMeta {
        name: format!("{fallback} (autosave)"),
        comment: String::new(),
    }
}

pub(crate) fn trim_autosave_dir(dir: &Path, keep: usize) -> std::io::Result<()> {
    let recordings = autosave_recordings_oldest_first(dir)?;
    let remove_count = recordings.len().saturating_sub(keep);
    for path in recordings.into_iter().take(remove_count) {
        remove_recording_with_meta(&path);
    }
    Ok(())
}

fn autosave_recordings_oldest_first(dir: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut recordings: Vec<(SystemTime, PathBuf)> = fs::read_dir(dir)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| is_recording_file(path))
        .map(|path| (modified_time(&path), path))
        .collect();
    recordings.sort();
    Ok(recordings.into_iter().map(|(_, path)| path).collect())
}

fn is_recording_file(path: &Path) -> bool {
    path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some(RECORDING_EXTENSION)
}

fn modified_time(path: &Path) -> SystemTime {
    fs::metadata(path)
        .and_then(|meta| meta.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH)
}

fn remove_recording_with_meta(path: &Path) {
    let _ = fs::remove_file(path);
    let _ = fs::remove_file(meta_path(path));
}
