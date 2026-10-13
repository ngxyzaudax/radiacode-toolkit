use std::fs;
use std::path::Path;

use crate::spectrogram::library_autosave::{
    AUTOSAVE_RETAIN, auto_save_snapshot, trim_autosave_dir,
};
use crate::spectrogram::library_meta::{load_meta, meta_path};
use crate::spectrogram::storage::{RecordingWriter, header_now, load_recording};
use crate::spectrogram::test_fixtures::{dir_string, write_fixture_recording};

fn recording_count(dir: &Path) -> usize {
    fs::read_dir(dir)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().and_then(|ext| ext.to_str()) == Some("rcwf"))
        .count()
}

fn touch_order(dir: &Path, names: &[&str]) {
    for (index, name) in names.iter().enumerate() {
        let path = write_fixture_recording(dir, name, &[vec![index as u32, 0, 0]]);
        fs::write(meta_path(&path), "{}").unwrap();
        let time = std::time::SystemTime::UNIX_EPOCH
            + std::time::Duration::from_secs(1_000 + index as u64);
        fs::File::open(&path).unwrap().set_modified(time).unwrap();
    }
}

#[test]
fn trimming_removes_oldest_recordings_with_their_sidecars() {
    let dir = tempfile::tempdir().unwrap();
    touch_order(dir.path(), &["a", "b", "c", "d", "e"]);
    trim_autosave_dir(dir.path(), 2).unwrap();
    assert_eq!(recording_count(dir.path()), 2);
    assert!(!dir.path().join("a.rcwf").exists());
    assert!(!meta_path(&dir.path().join("a.rcwf")).exists());
    assert!(dir.path().join("d.rcwf").exists());
    assert!(dir.path().join("e.rcwf").exists());
    assert!(meta_path(&dir.path().join("e.rcwf")).exists());
}

#[test]
fn trimming_ignores_non_recording_files() {
    let dir = tempfile::tempdir().unwrap();
    touch_order(dir.path(), &["a"]);
    fs::write(dir.path().join("notes.txt"), "x").unwrap();
    trim_autosave_dir(dir.path(), 1).unwrap();
    assert!(dir.path().join("a.rcwf").exists());
    assert!(dir.path().join("notes.txt").exists());
}

#[test]
fn snapshot_writes_series_and_names_after_active_recording() {
    let dir = tempfile::tempdir().unwrap();
    let source = write_fixture_recording(dir.path(), "source", &[vec![1, 2, 3]]);
    let series = load_recording(&source).unwrap();
    let header = header_now(0.0, 1.0, 0.0, 3, 10.0, None, vec![10.0, 20.0, 30.0]);
    let writer = RecordingWriter::create(dir.path().join("live.rcwf"), &header).unwrap();
    let path = auto_save_snapshot(&series, Some(&writer), &dir_string(dir.path())).unwrap();
    assert!(path.starts_with(dir.path().join("autosave")));
    assert_eq!(load_recording(&path).unwrap().rows.len(), 1);
    assert_eq!(load_meta(&path, "x").name, "live (autosave)");
    let unnamed = auto_save_snapshot(&series, None, &dir_string(dir.path())).unwrap();
    assert_ne!(unnamed, path);
    assert!(!meta_path(&unnamed).exists());
    assert!(recording_count(&dir.path().join("autosave")) <= AUTOSAVE_RETAIN);
}
