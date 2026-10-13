use egui::{Pos2, Rect, Response, Ui, Vec2};
use tracing::debug;

use crate::plot_zoom::{pinch_to_span_factor, scroll_y_to_span_factor};
use crate::spectrogram::layout::SpectrogramLayout;
use crate::spectrogram::state::SpectrogramState;
use crate::spectrogram::view_range::SpectrogramViewRange;

pub use crate::spectrogram::view_interact_hover::hover_details;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewGesture {
    pub hovered: bool,
    pub hover_pos: Option<Pos2>,
    pub scroll_y: f32,
    pub zoom_delta: f32,
    pub drag_delta: Vec2,
    pub dragged: bool,
}

pub fn handle_view_interaction(
    ui: &Ui,
    response: &Response,
    grid_rect: Rect,
    image_rect: Rect,
    layout: SpectrogramLayout,
    state: &mut SpectrogramState,
    total_rows: usize,
) {
    if response.double_clicked() {
        state.reset_view();
        return;
    }
    let gesture = read_gesture(ui, response);
    let changed = apply_gesture(
        &gesture,
        grid_rect,
        image_rect,
        layout,
        &mut state.view_range,
        total_rows,
    );
    if changed {
        state.texture.dirty = true;
    }
}

fn read_gesture(ui: &Ui, response: &Response) -> ViewGesture {
    let (scroll, zoom_delta) = ui.input(|input| (input.smooth_scroll_delta, input.zoom_delta()));
    ViewGesture {
        hovered: response.hovered(),
        hover_pos: response.hover_pos(),
        scroll_y: scroll.y,
        zoom_delta,
        drag_delta: response.drag_delta(),
        dragged: response.dragged(),
    }
}

pub fn apply_gesture(
    gesture: &ViewGesture,
    grid_rect: Rect,
    image_rect: Rect,
    layout: SpectrogramLayout,
    view_range: &mut SpectrogramViewRange,
    total_rows: usize,
) -> bool {
    let zoomed = apply_zoom(gesture, image_rect, view_range);
    let dragged = gesture.dragged
        && apply_drag(
            gesture, grid_rect, image_rect, layout, view_range, total_rows,
        );
    zoomed || dragged
}

fn apply_zoom(
    gesture: &ViewGesture,
    image_rect: Rect,
    view_range: &mut SpectrogramViewRange,
) -> bool {
    if !gesture.hovered {
        return false;
    }
    let mut changed = false;
    if (gesture.zoom_delta - 1.0).abs() > 0.001 {
        let factor = pinch_to_span_factor(gesture.zoom_delta);
        zoom_at_pointer(gesture.hover_pos, image_rect, view_range, factor);
        changed = true;
    }
    if gesture.scroll_y.abs() > 0.0 {
        let factor = scroll_y_to_span_factor(gesture.scroll_y);
        zoom_at_pointer(gesture.hover_pos, image_rect, view_range, factor);
        debug!(
            factor,
            energy_min = view_range.energy_min_kev,
            energy_max = view_range.energy_max_kev,
            "spectrogram energy zoom"
        );
        changed = true;
    }
    changed
}

fn apply_drag(
    gesture: &ViewGesture,
    grid_rect: Rect,
    image_rect: Rect,
    layout: SpectrogramLayout,
    view_range: &mut SpectrogramViewRange,
    total_rows: usize,
) -> bool {
    let delta = gesture.drag_delta;
    if !view_range.fit_full_spectrum && delta.x.abs() > 0.0 {
        view_range.pan_energy(pan_delta_kev(delta.x, image_rect, view_range));
    }
    let pointer_in_grid = gesture
        .hover_pos
        .is_some_and(|position| grid_rect.contains(position));
    if pointer_in_grid && layout.cell_px > 0.0 {
        let row_delta = (-delta.y / layout.cell_px).round() as i32;
        if row_delta != 0 {
            view_range.scroll_history(row_delta, total_rows, layout.display_rows);
        }
    }
    true
}

fn pan_delta_kev(drag_x: f32, image_rect: Rect, view_range: &SpectrogramViewRange) -> f64 {
    let span = view_range.energy_max_kev - view_range.energy_min_kev;
    let width = image_rect.width().max(1.0) as f64;
    -(drag_x as f64 / width) * span
}

fn zoom_at_pointer(
    hover_pos: Option<Pos2>,
    image_rect: Rect,
    view_range: &mut SpectrogramViewRange,
    factor: f64,
) {
    let anchor = hover_pos
        .map(|pos| energy_at_x(pos.x, image_rect, view_range))
        .unwrap_or_else(|| 0.5 * (view_range.energy_min_kev + view_range.energy_max_kev));
    view_range.zoom_energy(anchor, factor);
}

fn energy_at_x(x: f32, image_rect: Rect, view_range: &SpectrogramViewRange) -> f64 {
    let t = ((x - image_rect.left()) / image_rect.width().max(1.0)).clamp(0.0, 1.0) as f64;
    view_range.energy_min_kev + (view_range.energy_max_kev - view_range.energy_min_kev) * t
}
