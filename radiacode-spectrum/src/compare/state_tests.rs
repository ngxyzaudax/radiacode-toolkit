use crate::compare::selection::selection_status;
use crate::compare::state::CompareState;
use crate::spectrogram::model::RecordingEntry;
use crate::spectrogram::test_fixtures::{dir_string, write_fixture_recording};

struct Library {
    state: CompareState,
    _dir: tempfile::TempDir,
}

fn library_with(names: &[&str]) -> Library {
    let dir = tempfile::tempdir().unwrap();
    for (index, name) in names.iter().enumerate() {
        let scale = (index + 1) as u32;
        write_fixture_recording(
            dir.path(),
            name,
            &[vec![scale, scale * 2, scale * 3], vec![scale, scale, scale]],
        );
    }
    let mut state = CompareState::new();
    state.refresh_library(&dir_string(dir.path()));
    Library { state, _dir: dir }
}

fn entry(state: &CompareState, name: &str) -> RecordingEntry {
    state
        .library
        .iter()
        .find(|entry| entry.name == name)
        .cloned()
        .expect("library entry")
}

#[test]
fn background_and_samples_are_mutually_exclusive() {
    let mut lib = library_with(&["bg", "s1"]);
    let bg = entry(&lib.state, "bg");
    let s1 = entry(&lib.state, "s1");
    lib.state.set_background(&bg);
    assert!(lib.state.is_background(&bg.path));
    lib.state.toggle_sample(&bg);
    assert!(lib.state.sample_index(&bg.path).is_none());
    lib.state.toggle_sample(&s1);
    lib.state.set_background(&s1);
    assert!(lib.state.is_background(&bg.path));
    assert_eq!(lib.state.sample_index(&s1.path), Some(0));
}

#[test]
fn selecting_background_and_sample_computes_subtraction() {
    let mut lib = library_with(&["bg", "s1"]);
    let bg = entry(&lib.state, "bg");
    let s1 = entry(&lib.state, "s1");
    lib.state.toggle_sample(&s1);
    assert_eq!(lib.state.status, selection_status(false, 1));
    assert!(lib.state.samples[0].subtraction.is_none());
    lib.state.set_background(&bg);
    assert_eq!(lib.state.status, "Comparison ready.");
    let subtraction = lib.state.samples[0].subtraction.as_ref().unwrap();
    assert!((subtraction.scale_factor - 1.0).abs() < 1e-9);
    assert_eq!(subtraction.net_counts, vec![2.0, 3.0, 4.0]);
    assert!(lib.state.error.is_empty());
}

#[test]
fn toggling_and_removing_samples_updates_selection() {
    let mut lib = library_with(&["s1", "s2"]);
    let s1 = entry(&lib.state, "s1");
    let s2 = entry(&lib.state, "s2");
    lib.state.toggle_sample(&s1);
    lib.state.toggle_sample(&s2);
    assert_eq!(
        lib.state.status,
        "2 samples selected. Add a background to compute net."
    );
    lib.state.toggle_sample(&s1);
    assert_eq!(lib.state.sample_paths, vec![s2.path.clone()]);
    lib.state.remove_sample_at(5);
    assert_eq!(lib.state.samples.len(), 1);
    lib.state.remove_sample_at(0);
    assert!(lib.state.samples.is_empty());
    assert!(lib.state.status.is_empty());
}

#[test]
fn clearing_background_drops_subtractions() {
    let mut lib = library_with(&["bg", "s1"]);
    let bg = entry(&lib.state, "bg");
    let s1 = entry(&lib.state, "s1");
    lib.state.set_background(&bg);
    lib.state.toggle_sample(&s1);
    lib.state.clear_background();
    assert!(lib.state.background.is_none());
    assert!(lib.state.samples[0].subtraction.is_none());
    assert_eq!(lib.state.status, selection_status(false, 1));
}

#[test]
fn missing_recording_is_reported_as_error() {
    let mut lib = library_with(&["s1"]);
    let s1 = entry(&lib.state, "s1");
    std::fs::remove_file(&s1.path).unwrap();
    lib.state.toggle_sample(&s1);
    assert!(lib.state.samples.is_empty());
    assert_eq!(lib.state.error, "Failed to load a selected sample.");
}

#[test]
fn library_filter_matches_names_case_insensitively() {
    let mut lib = library_with(&["Alpha", "beta"]);
    lib.state.library_filter = "ALP".into();
    let filtered = lib.state.filtered_library();
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].name, "Alpha");
}
