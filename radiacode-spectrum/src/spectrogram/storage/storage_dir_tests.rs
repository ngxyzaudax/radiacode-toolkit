use std::fs;

use crate::persist::json_store::use_test_data_dir;
use crate::spectrogram::storage::storage_dir::{
    default_spectrograms_dir, ensure_dir, list_recordings, spectrograms_dir, timestamp_filename,
};
use crate::spectrogram::test_fixtures::write_fixture_recording;

#[test]
fn blank_configuration_resolves_to_default_data_dir() {
    let data = tempfile::tempdir().unwrap();
    use_test_data_dir(data.path().to_path_buf());
    assert_eq!(default_spectrograms_dir(), data.path().join("spectrograms"));
    assert_eq!(spectrograms_dir("  "), data.path().join("spectrograms"));
    assert_eq!(
        spectrograms_dir(" /tmp/custom "),
        std::path::PathBuf::from("/tmp/custom")
    );
}

#[test]
fn ensure_dir_migrates_legacy_waterfall_recordings_once() {
    let data = tempfile::tempdir().unwrap();
    use_test_data_dir(data.path().to_path_buf());
    let legacy = data.path().join("waterfalls");
    fs::create_dir_all(&legacy).unwrap();
    write_fixture_recording(&legacy, "old", &[vec![1, 2, 3]]);
    fs::write(legacy.join("notes.txt"), b"ignored").unwrap();
    let dir = ensure_dir("").unwrap();
    assert_eq!(dir, data.path().join("spectrograms"));
    assert!(dir.join("old.rcwf").exists());
    assert!(!dir.join("notes.txt").exists());
    fs::write(dir.join("old.rcwf"), b"replaced").unwrap();
    ensure_dir("").unwrap();
    assert_eq!(fs::read(dir.join("old.rcwf")).unwrap(), b"replaced");
}

#[test]
fn list_recordings_skips_foreign_and_corrupt_files() {
    let dir = tempfile::tempdir().unwrap();
    write_fixture_recording(dir.path(), "good", &[vec![1, 2, 3]]);
    fs::write(dir.path().join("bad.rcwf"), b"garbage").unwrap();
    fs::write(dir.path().join("other.txt"), b"text").unwrap();
    let entries = list_recordings(&dir.path().display().to_string()).unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].name, "good");
    assert_eq!(entries[0].channel_count, 3);
    assert_eq!(entries[0].device_serial.as_deref(), Some("SN-FIXTURE"));
}

#[test]
fn timestamp_filename_has_recording_extension() {
    let name = timestamp_filename();
    assert!(name.ends_with(".rcwf"));
    assert_eq!(name.len(), "2026-01-01_00-00-00.rcwf".len());
}
