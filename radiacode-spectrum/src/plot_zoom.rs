use egui_plot::PlotUi;

use crate::energy::{ENERGY_MAX_KEV, ENERGY_MIN_KEV, clamp_energy_range};

pub const ZOOM_IN_FACTOR: f64 = 0.85;
pub const ZOOM_OUT_FACTOR: f64 = 1.18;
pub const FIT_FULL_THRESHOLD: f64 = 0.98;

pub fn scroll_y_to_span_factor(scroll_y: f32) -> f64 {
    if scroll_y > 0.0 {
        ZOOM_IN_FACTOR
    } else {
        ZOOM_OUT_FACTOR
    }
}

pub fn pinch_to_span_factor(zoom_delta: f32) -> f64 {
    (1.0 / zoom_delta as f64).clamp(ZOOM_IN_FACTOR, ZOOM_OUT_FACTOR)
}

pub fn zoom_energy_span(min_x: f64, max_x: f64, anchor: f64, factor: f64) -> (f64, f64) {
    let span = (max_x - min_x).max(1.0);
    let new_span = (span * factor).max(1.0);
    let ratio = ((anchor - min_x) / span).clamp(0.0, 1.0);
    let min = anchor - new_span * ratio;
    (min, min + new_span)
}

pub fn apply_energy_axis_navigation(plot_ui: &mut PlotUi) -> (f64, f64) {
    if plot_ui.response().double_clicked() {
        plot_ui.set_plot_bounds_x(ENERGY_MIN_KEV..=ENERGY_MAX_KEV);
        return (ENERGY_MIN_KEV, ENERGY_MAX_KEV);
    }
    let bounds = plot_ui.plot_bounds();
    let mut min_x = bounds.min()[0];
    let mut max_x = bounds.max()[0];
    if plot_ui.response().hovered() {
        let (scroll_y, zoom_delta) = plot_ui
            .ctx()
            .input(|input| (input.smooth_scroll_delta.y, input.zoom_delta()));
        if let Some(factor) = energy_zoom_factor(scroll_y, zoom_delta) {
            let anchor = plot_ui
                .pointer_coordinate()
                .map(|point| point.x)
                .unwrap_or((min_x + max_x) * 0.5);
            (min_x, max_x) = zoom_energy_span(min_x, max_x, anchor, factor);
        }
    }
    let (min_x, max_x) = snap_energy_range_if_near_full(min_x, max_x);
    plot_ui.set_plot_bounds_x(min_x..=max_x);
    (min_x, max_x)
}

fn energy_zoom_factor(scroll_y: f32, zoom_delta: f32) -> Option<f64> {
    if scroll_y.abs() > 0.0 {
        Some(scroll_y_to_span_factor(scroll_y))
    } else if (zoom_delta - 1.0).abs() > 0.001 {
        Some(pinch_to_span_factor(zoom_delta))
    } else {
        None
    }
}

pub fn snap_energy_range_if_near_full(min_x: f64, max_x: f64) -> (f64, f64) {
    let full_span = ENERGY_MAX_KEV - ENERGY_MIN_KEV;
    let width = max_x - min_x;
    if width >= full_span * FIT_FULL_THRESHOLD {
        return (ENERGY_MIN_KEV, ENERGY_MAX_KEV);
    }
    clamp_energy_range(min_x, max_x)
}

#[cfg(test)]
mod tests {
    use super::{ZOOM_IN_FACTOR, ZOOM_OUT_FACTOR, scroll_y_to_span_factor, zoom_energy_span};

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
}
