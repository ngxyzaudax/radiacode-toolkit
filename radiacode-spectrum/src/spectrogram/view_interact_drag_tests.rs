use egui::{Pos2, Rect, Vec2, pos2};

use crate::spectrogram::color_scheme::ColorScheme;
use crate::spectrogram::layout::SpectrogramLayout;
use crate::spectrogram::view_interact::{ViewGesture, apply_gesture};
use crate::spectrogram::view_interact_hover::hover_cell;
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

fn zoomed_range() -> SpectrogramViewRange {
    let mut range = SpectrogramViewRange::new();
    range.zoom_energy(1500.0, 0.5);
    range
}

#[test]
fn horizontal_drag_pans_energy_only_when_zoomed() {
    let gesture = ViewGesture {
        dragged: true,
        drag_delta: Vec2::new(-20.0, 0.0),
        ..idle_gesture()
    };
    let mut full = SpectrogramViewRange::new();
    apply_gesture(
        &gesture,
        image_rect(),
        image_rect(),
        layout(),
        &mut full,
        50,
    );
    assert_eq!(full.energy_min_kev, 0.0);
    let mut zoomed = zoomed_range();
    let before = zoomed.energy_min_kev;
    apply_gesture(
        &gesture,
        image_rect(),
        image_rect(),
        layout(),
        &mut zoomed,
        50,
    );
    assert!(zoomed.energy_min_kev > before);
}

#[test]
fn vertical_drag_scrolls_history_rows() {
    let gesture = ViewGesture {
        dragged: true,
        drag_delta: Vec2::new(0.0, 30.0),
        ..idle_gesture()
    };
    let mut range = SpectrogramViewRange::new();
    apply_gesture(
        &gesture,
        image_rect(),
        image_rect(),
        layout(),
        &mut range,
        50,
    );
    assert!(!range.follow_live);
    assert_eq!(range.row_start, 37);
}

#[test]
fn hover_cell_maps_position_to_column_and_row() {
    let cols = [4usize, 5, 6, 7];
    let cell = hover_cell(pos2(150.0, 99.0), image_rect(), 4, &cols).unwrap();
    assert_eq!(cell.source_col, 7);
    assert_eq!(cell.row_index, 3);
    assert!(hover_cell(Pos2::new(500.0, 10.0), image_rect(), 4, &cols).is_none());
    assert!(hover_cell(pos2(10.0, 10.0), image_rect(), 0, &cols).is_none());
    assert!(hover_cell(pos2(10.0, 10.0), image_rect(), 4, &[]).is_none());
}

#[test]
fn history_scrolling_clamps_and_resumes_follow_at_end() {
    let mut range = SpectrogramViewRange::new();
    assert_eq!(range.visible_start(100, 10), 90);
    range.scroll_history(-5, 100, 10);
    assert_eq!(range.row_start, 85);
    assert!(!range.follow_live);
    range.scroll_history(-500, 100, 10);
    assert_eq!(range.row_start, 0);
    range.scroll_history(90, 100, 10);
    assert!(range.follow_live);
    range.scroll_history(-3, 5, 10);
    assert!(range.follow_live);
    assert_eq!(range.row_start, 0);
}

#[test]
fn clamp_to_history_keeps_start_within_bounds() {
    let mut range = SpectrogramViewRange::new();
    range.scroll_history(-50, 100, 10);
    range.clamp_to_history(40, 10);
    assert_eq!(range.row_start, 30);
    range.clamp_to_history(8, 10);
    assert!(range.follow_live);
    assert_eq!(range.row_start, 0);
}

#[test]
fn color_scheme_round_trips_and_falls_back_to_viridis() {
    for scheme in ColorScheme::ALL {
        let json = serde_json::to_string(&scheme).unwrap();
        let parsed: ColorScheme = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, scheme);
    }
    let unknown: ColorScheme = serde_json::from_str("\"Magma\"").unwrap();
    assert_eq!(unknown, ColorScheme::Viridis);
    assert_eq!(ColorScheme::Turbo.label(), "Turbo");
}
