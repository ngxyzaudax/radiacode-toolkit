use crate::spectrogram::library::{
    delete_entry, export_rcspg, import_rcspg, rename_entry, set_comment,
};
use crate::spectrogram::library_meta::{load_meta, meta_path};
use crate::spectrogram::storage::{list_recordings, load_recording};
use crate::spectrogram::test_fixtures::{dir_string, write_fixture_recording};

#[test]
fn rename_and_comment_persist_in_sidecar_meta() {
    let dir = tempfile::tempdir().unwrap();
    let path = write_fixture_recording(dir.path(), "first", &[vec![1, 2, 3]]);
    rename_entry(&path, "Renamed").unwrap();
    set_comment(&path, "Some notes").unwrap();
    let meta = load_meta(&path, "fallback");
    assert_eq!(meta.name, "Renamed");
    assert_eq!(meta.comment, "Some notes");
    let entries = list_recordings(&dir_string(dir.path())).unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].name, "Renamed");
    assert_eq!(entries[0].comment, "Some notes");
    assert_eq!(entries[0].row_count, 1);
}

#[test]
fn comment_keeps_file_stem_as_name_when_no_meta_exists() {
    let dir = tempfile::tempdir().unwrap();
    let path = write_fixture_recording(dir.path(), "stem-name", &[vec![1, 2, 3]]);
    set_comment(&path, "note").unwrap();
    assert_eq!(load_meta(&path, "other").name, "stem-name");
}

#[test]
fn delete_removes_recording_and_meta() {
    let dir = tempfile::tempdir().unwrap();
    let path = write_fixture_recording(dir.path(), "doomed", &[vec![1, 2, 3]]);
    rename_entry(&path, "Doomed").unwrap();
    assert!(meta_path(&path).exists());
    delete_entry(&path).unwrap();
    assert!(!path.exists());
    assert!(!meta_path(&path).exists());
    assert!(delete_entry(&path).is_err());
}

#[test]
fn export_then_import_round_trips_rows_and_name() {
    let dir = tempfile::tempdir().unwrap();
    let path = write_fixture_recording(dir.path(), "source", &[vec![1, 2, 3], vec![4, 5, 6]]);
    rename_entry(&path, "Exported").unwrap();
    let exported = dir.path().join("exported.rcspg");
    export_rcspg(&path, &exported).unwrap();
    let import_dir = tempfile::tempdir().unwrap();
    let imported = import_rcspg(&exported, &dir_string(import_dir.path())).unwrap();
    let series = load_recording(&imported).unwrap();
    assert_eq!(series.rows.len(), 2);
    assert_eq!(series.rows[1].counts, vec![4, 5, 6]);
    assert_eq!(load_meta(&imported, "x").name, "exported");
}

#[test]
fn export_of_missing_recording_reports_error() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("missing.rcwf");
    let destination = dir.path().join("out.rcspg");
    assert!(export_rcspg(&missing, &destination).is_err());
}
