use std::fs;
use std::path::{Path, PathBuf};

use crate::spectrogram::library_meta::{LibraryMeta, load_meta, save_meta};
use crate::spectrogram::model::SpectrogramSeries;
use crate::spectrogram::rcspg;
use crate::spectrogram::storage::{
    RecordingWriter, ensure_dir, load_recording, spectrograms_dir, timestamp_filename,
    write_recording,
};

pub fn rename_entry(path: &Path, name: &str) -> Result<(), String> {
    let mut meta = load_meta(path, name);
    meta.name = name.to_string();
    save_meta(path, &meta).map_err(|error| error.to_string())
}

pub fn set_comment(path: &Path, comment: &str) -> Result<(), String> {
    let fallback = path
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("recording");
    let mut meta = load_meta(path, fallback);
    meta.comment = comment.to_string();
    save_meta(path, &meta).map_err(|error| error.to_string())
}

pub fn delete_entry(path: &Path) -> Result<(), String> {
    fs::remove_file(path).map_err(|error| error.to_string())?;
    let meta_path = crate::spectrogram::library_meta::meta_path(path);
    if meta_path.exists() {
        let _ = fs::remove_file(meta_path);
    }
    Ok(())
}

pub fn export_rcspg(path: &Path, destination: &Path) -> Result<(), String> {
    let series = load_recording(path).map_err(|error| error.to_string())?;
    let meta = load_meta(
        path,
        path.file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("recording"),
    );
    rcspg::export_recording(destination, &series, &meta.name, &meta.comment)
        .map_err(|error| error.to_string())
}

pub fn import_rcspg(source: &Path, recordings_dir: &str) -> Result<PathBuf, String> {
    let series = rcspg::import_recording(source).map_err(|error| error.to_string())?;
    let dir = ensure_dir(recordings_dir).map_err(|error| error.to_string())?;
    let path = dir.join(timestamp_filename());
    write_recording(&path, &series).map_err(|error| error.to_string())?;
    let meta = LibraryMeta {
        name: source
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("import")
            .to_string(),
        comment: String::new(),
    };
    save_meta(&path, &meta).map_err(|error| error.to_string())?;
    Ok(path)
}

const AUTOSAVE_RETAIN: usize = 3;

pub fn auto_save_snapshot(
    series: &SpectrogramSeries,
    writer: Option<&RecordingWriter>,
    recordings_dir: &str,
) -> std::io::Result<PathBuf> {
    let dir = spectrograms_dir(recordings_dir).join("autosave");
    fs::create_dir_all(&dir)?;
    trim_autosave_dir(&dir, AUTOSAVE_RETAIN)?;
    let path = dir.join(format!("autosave_{}", timestamp_filename()));
    write_recording(&path, series)?;
    if let Some(writer) = writer {
        let fallback = writer
            .path
            .file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("recording");
        let _ = save_meta(
            &path,
            &LibraryMeta {
                name: format!("{fallback} (autosave)"),
                comment: String::new(),
            },
        );
    }
    Ok(path)
}

fn trim_autosave_dir(dir: &Path, keep: usize) -> std::io::Result<()> {
    let mut entries: Vec<_> = fs::read_dir(dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_file())
        .collect();
    if entries.len() <= keep {
        return Ok(());
    }
    entries.sort_by_key(|entry| entry.metadata().and_then(|meta| meta.modified()).ok());
    let remove_count = entries.len().saturating_sub(keep);
    for entry in entries.into_iter().take(remove_count) {
        let _ = fs::remove_file(entry.path());
    }
    Ok(())
}
