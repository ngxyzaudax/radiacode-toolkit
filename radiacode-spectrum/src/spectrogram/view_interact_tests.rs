use egui::{Rect, Vec2, pos2};

use crate::spectrogram::layout::SpectrogramLayout;
use crate::spectrogram::view_interact::{ViewGesture, apply_gesture};

use crate::spectrogram::view_range::SpectrogramViewRange;

fn image_rect() -> Rect {
    Rect::from_min_max(pos2(0.0, 0.0), pos2(200.0, 100.0))
}

fn layout() -> SpectrogramLayout {
    SpectrogramLayout {
        cell_px: 10.0,
        display_cols: 20,
        display_rows: 10,
        image_rect: image_rect(),
    }
}

fn idle_gesture() -> ViewGesture {
    ViewGesture {
        hovered: true,
        hover_pos: Some(pos2(100.0, 50.0)),
        scroll_y: 0.0,
        zoom_delta: 1.0,
        drag_delta: Vec2::ZERO,
        dragged: false,
    }
}

#[test]
fn idle_gesture_changes_nothing() {
    let mut range = SpectrogramViewRange::new();
    let changed = apply_gesture(
        &idle_gesture(),
        image_rect(),
        image_rect(),
        layout(),
        &mut range,
        50,
    );
    assert!(!changed);
    assert!(range.fit_full_spectrum);
}

#[test]
fn scroll_zooms_around_pointer_energy() {
    let mut range = SpectrogramViewRange::new();
    let gesture = ViewGesture {
        scroll_y: 1.0,
        hover_pos: Some(pos2(0.0, 50.0)),
        ..idle_gesture()
    };
    let changed = apply_gesture(
        &gesture,
        image_rect(),
        image_rect(),
        layout(),
        &mut range,
        50,
    );
    assert!(changed);
    assert!(!range.fit_full_spectrum);
    assert_eq!(range.energy_min_kev, 0.0);
    assert!(range.energy_max_kev < 3000.0);
}

#[test]
fn zoom_is_ignored_when_not_hovered() {
    let mut range = SpectrogramViewRange::new();
    let gesture = ViewGesture {
        hovered: false,
        scroll_y: 1.0,
        ..idle_gesture()
    };
    assert!(!apply_gesture(
        &gesture,
        image_rect(),
        image_rect(),
        layout(),
        &mut range,
        50
    ));
}
