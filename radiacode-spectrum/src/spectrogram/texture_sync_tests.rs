use std::sync::{Arc, Mutex};

use egui::{Context, Rect, pos2};

use crate::persist::json_store::use_test_data_dir;
use crate::spectrogram::capture::SpectrogramCapture;
use crate::spectrogram::layout::SpectrogramLayout;
use crate::spectrogram::state::SpectrogramState;
use crate::spectrogram::test_fixtures::write_fixture_recording;
use crate::spectrogram::texture_sync::{sync_texture, texture_size};

fn layout(display_cols: usize, display_rows: usize) -> SpectrogramLayout {
    SpectrogramLayout {
        cell_px: 4.0,
        display_cols,
        display_rows,
        image_rect: Rect::from_min_max(pos2(0.0, 0.0), pos2(100.0, 100.0)),
    }
}

fn state_with_loaded_recording() -> (SpectrogramState, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    use_test_data_dir(dir.path().to_path_buf());
    let capture = Arc::new(Mutex::new(SpectrogramCapture::new()));
    let mut state = SpectrogramState::new(capture);
    let path = write_fixture_recording(dir.path(), "tex", &[vec![1, 2, 3], vec![4, 5, 6]]);
    state.request_load(path);
    (state, dir)
}

#[test]
fn texture_size_falls_back_to_layout_when_no_columns() {
    assert_eq!(texture_size(&[], layout(0, 0)), (1, 1));
    assert_eq!(texture_size(&[], layout(12, 7)), (12, 7));
    assert_eq!(texture_size(&[3, 4], layout(12, 7)), (2, 7));
}

#[test]
fn sync_without_series_creates_blank_texture_of_layout_size() {
    let dir = tempfile::tempdir().unwrap();
    use_test_data_dir(dir.path().to_path_buf());
    let capture = Arc::new(Mutex::new(SpectrogramCapture::new()));
    let mut state = SpectrogramState::new(capture);
    state.texture.dirty = true;
    sync_texture(&Context::default(), &mut state, layout(8, 5));
    assert_eq!(state.texture.image.width(), 8);
    assert_eq!(state.texture.image.height(), 5);
    assert!(!state.texture.dirty);
    assert!(state.texture_handle.is_some());
}

#[test]
fn sync_with_loaded_series_rebuilds_from_visible_rows() {
    let (mut state, _dir) = state_with_loaded_recording();
    sync_texture(&Context::default(), &mut state, layout(3, 4));
    assert_eq!(state.texture.image.width(), 3);
    assert_eq!(state.texture.image.height(), 4);
    assert!(state.z_range.is_some());
    assert_eq!(state.z_range_rows, 2);
    assert!(!state.texture.dirty);
    sync_texture(&Context::default(), &mut state, layout(3, 6));
    assert_eq!(state.texture.image.height(), 6);
    assert!(!state.texture.dirty);
}
