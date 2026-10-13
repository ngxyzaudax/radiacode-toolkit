use super::{
    FIT_FULL_THRESHOLD, ZOOM_IN_FACTOR, ZOOM_OUT_FACTOR, energy_zoom_factor, pinch_to_span_factor,
    scroll_y_to_span_factor, snap_energy_range_if_near_full, zoom_energy_span,
};
use crate::energy::{ENERGY_MAX_KEV, ENERGY_MIN_KEV};

#[test]
fn scroll_up_zooms_in() {
    assert_eq!(scroll_y_to_span_factor(1.0), ZOOM_IN_FACTOR);
    let (min, max) = zoom_energy_span(0.0, 1000.0, 500.0, ZOOM_IN_FACTOR);
    assert!((max - min - 850.0).abs() < 0.01);
}

#[test]
fn scroll_down_zooms_out() {
    assert_eq!(scroll_y_to_span_factor(-1.0), ZOOM_OUT_FACTOR);
    let (min, max) = zoom_energy_span(100.0, 200.0, 150.0, ZOOM_OUT_FACTOR);
    assert!(max - min > 100.0);
}

#[test]
fn zoom_keeps_anchor_ratio_and_minimum_span() {
    let (min, max) = zoom_energy_span(0.0, 1000.0, 0.0, ZOOM_IN_FACTOR);
    assert_eq!(min, 0.0);
    assert!((max - 850.0).abs() < 0.01);
    let (min, max) = zoom_energy_span(10.0, 10.5, 10.2, 0.1);
    assert!((max - min - 1.0).abs() < 1e-9);
}

#[test]
fn pinch_factor_is_inverse_and_clamped() {
    assert_eq!(pinch_to_span_factor(1.0), 1.0);
    assert_eq!(pinch_to_span_factor(10.0), ZOOM_IN_FACTOR);
    assert_eq!(pinch_to_span_factor(0.1), ZOOM_OUT_FACTOR);
}

#[test]
fn zoom_factor_prefers_scroll_then_pinch() {
    assert_eq!(energy_zoom_factor(2.0, 0.5), Some(ZOOM_IN_FACTOR));
    assert_eq!(energy_zoom_factor(0.0, 0.5), Some(ZOOM_OUT_FACTOR));
    assert_eq!(energy_zoom_factor(0.0, 1.0), None);
    assert_eq!(energy_zoom_factor(0.0, 1.0005), None);
}

#[test]
fn near_full_ranges_snap_to_full_axis() {
    let span = ENERGY_MAX_KEV - ENERGY_MIN_KEV;
    let near_full = span * FIT_FULL_THRESHOLD + 1.0;
    assert_eq!(
        snap_energy_range_if_near_full(5.0, 5.0 + near_full),
        (ENERGY_MIN_KEV, ENERGY_MAX_KEV)
    );
    let (min, max) = snap_energy_range_if_near_full(-50.0, 450.0);
    assert_eq!(min, ENERGY_MIN_KEV);
    assert_eq!(max, 500.0);
}
