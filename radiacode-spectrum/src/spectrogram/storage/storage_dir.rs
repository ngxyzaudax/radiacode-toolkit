use std::fs::{self, File};
use std::path::{Path, PathBuf};

use directories::ProjectDirs;
use time::OffsetDateTime;

use crate::spectrogram::library_meta::load_meta;
use crate::spectrogram::model::{RecordingEntry, SpectrogramHeader};

use super::storage_format::read_recording_prefix;

pub fn default_spectrograms_dir() -> PathBuf {
    ProjectDirs::from("com", "radiacode", "radiacode-spectrum")
        .map(|dirs| dirs.data_dir().join("spectrograms"))
        .unwrap_or_else(|| PathBuf::from("spectrograms"))
}

pub fn spectrograms_dir(configured: &str) -> PathBuf {
    let trimmed = configured.trim();
    if trimmed.is_empty() {
        default_spectrograms_dir()
    } else {
        PathBuf::from(trimmed)
    }
}

fn legacy_spectrograms_dir() -> PathBuf {
    ProjectDirs::from("com", "radiacode", "radiacode-spectrum")
        .map(|dirs| dirs.data_dir().join("waterfalls"))
        .unwrap_or_else(|| PathBuf::from("waterfalls"))
}

pub fn ensure_dir(configured: &str) -> std::io::Result<PathBuf> {
    let dir = spectrograms_dir(configured);
    fs::create_dir_all(&dir)?;
    if configured.trim().is_empty() {
        let legacy = legacy_spectrograms_dir();
        if legacy.exists() && legacy != dir {
            migrate_legacy_recordings(&legacy, &dir)?;
        }
    }
    Ok(dir)
}

fn migrate_legacy_recordings(from: &Path, to: &Path) -> std::io::Result<()> {
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("rcwf") {
            continue;
        }
        let Some(name) = path.file_name() else {
            continue;
        };
        let destination = to.join(name);
        if !destination.exists() {
            let _ = fs::copy(&path, &destination);
        }
    }
    Ok(())
}

use time::macros::format_description;

pub fn timestamp_filename() -> String {
    let now = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
    format!(
        "{}.rcwf",
        now.format(format_description!(
            "[year]-[month]-[day]_[hour]-[minute]-[second]"
        ))
        .unwrap_or_else(|_| "recording".into())
    )
}

pub fn list_recordings(configured: &str) -> std::io::Result<Vec<RecordingEntry>> {
    let dir = ensure_dir(configured)?;
    let mut entries = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("rcwf") {
            continue;
        }
        if let Some(item) = build_entry(path) {
            entries.push(item);
        }
    }
    entries.sort_by(|left, right| right.created_at.cmp(&left.created_at));
    Ok(entries)
}

pub struct RecordingIndex {
    pub header: SpectrogramHeader,
    pub row_count: u32,
}

pub fn load_recording_index(path: &Path) -> std::io::Result<RecordingIndex> {
    let mut file = File::open(path)?;
    let (version, header, _channel_count, row_count) = read_recording_prefix(&mut file)?;
    if version != super::storage_format::VERSION_V1 && version != super::storage_format::VERSION_V2
    {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "unsupported spectrogram file version",
        ));
    }
    Ok(RecordingIndex { header, row_count })
}

pub(crate) fn build_entry(path: PathBuf) -> Option<RecordingEntry> {
    let fallback = path
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("recording")
        .to_string();
    let index = load_recording_index(&path).ok()?;
    let meta = load_meta(&path, &fallback);
    Some(RecordingEntry {
        path,
        name: meta.name,
        comment: meta.comment,
        created_at: index.header.created_at.clone(),
        device_serial: index.header.device_serial.clone(),
        interval_secs: index.header.interval_secs,
        row_count: index.row_count,
        channel_count: index.header.channel_count,
    })
}
