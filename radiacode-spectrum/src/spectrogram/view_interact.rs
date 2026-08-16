use egui::Ui;
use tracing::debug;

use crate::plot_hover::{SpectrogramHoverInfo, spectrogram_hover_text};
use crate::spectrogram::layout::SpectrogramLayout;
use crate::spectrogram::model::SpectrogramSeries;
use crate::spectrogram::state::SpectrogramState;

pub fn handle_view_interaction(
    ui: &Ui,
    response: &egui::Response,
    grid_rect: egui::Rect,
    image_rect: egui::Rect,
    layout: SpectrogramLayout,
    state: &mut SpectrogramState,
    total_rows: usize,
) {
    if response.double_clicked() {
        state.reset_view();
        return;
    }

    let (scroll, zoom_delta, shift) = ui.input(|input| {
        (
            input.smooth_scroll_delta,
            input.zoom_delta(),
            input.modifiers.shift,
        )
    });
    let pointer_in_grid = response
        .hover_pos()
        .is_some_and(|position| grid_rect.contains(position));

    let mut changed = false;
    if response.hovered() && (zoom_delta - 1.0).abs() > 0.001 {
        let factor = (1.0 / zoom_delta as f64).clamp(0.5, 2.0);
        zoom_at_pointer(response, image_rect, state, factor);
        changed = true;
    }

    if response.hovered() && scroll.y.abs() > 0.0 {
        if shift && pointer_in_grid {
            let row_delta = if scroll.y > 0.0 { -1 } else { 1 };
            state
                .view_range
                .scroll_history(row_delta, total_rows, layout.display_rows);
            debug!(row_delta, "spectrogram history scroll");
        } else if !shift {
            let factor = if scroll.y > 0.0 { 0.85 } else { 1.18 };
            zoom_at_pointer(response, image_rect, state, factor);
            debug!(
                factor,
                energy_min = state.view_range.energy_min_kev,
                energy_max = state.view_range.energy_max_kev,
                "spectrogram energy zoom"
            );
        }
        changed = true;
    }

    if response.hovered() && scroll.x.abs() > 0.0 {
        let delta = if scroll.x > 0.0 { -1 } else { 1 };
        let channels = state
            .active_series()
            .map(|series| {
                crate::spectrogram::layout::channels_in_energy_range(
                    &series.energies_kev,
                    state.view_range.energy_min_kev,
                    state.view_range.energy_max_kev,
                )
            })
            .unwrap_or(1);
        state
            .view_range
            .scroll_channels(delta, channels, layout.display_cols);
        changed = true;
    }

    if response.dragged() {
        let delta = response.drag_delta();
        if delta.x.abs() >= delta.y.abs() {
            let span = state.view_range.energy_max_kev - state.view_range.energy_min_kev;
            let width = image_rect.width().max(1.0) as f64;
            let delta_kev = -(delta.x as f64 / width) * span;
            state.view_range.pan_energy(delta_kev);
        } else if pointer_in_grid && layout.cell_px > 0.0 {
            let row_delta = (-delta.y / layout.cell_px).round() as i32;
            if row_delta != 0 {
                state
                    .view_range
                    .scroll_history(row_delta, total_rows, layout.display_rows);
            }
        }
        changed = true;
    }

    if changed {
        state.texture.dirty = true;
    }
}

fn zoom_at_pointer(
    response: &egui::Response,
    image_rect: egui::Rect,
    state: &mut SpectrogramState,
    factor: f64,
) {
    let anchor = response
        .hover_pos()
        .map(|pos| {
            let t =
                ((pos.x - image_rect.left()) / image_rect.width().max(1.0)).clamp(0.0, 1.0) as f64;
            state.view_range.energy_min_kev
                + (state.view_range.energy_max_kev - state.view_range.energy_min_kev) * t
        })
        .unwrap_or_else(|| {
            0.5 * (state.view_range.energy_min_kev + state.view_range.energy_max_kev)
        });
    state.view_range.zoom_energy(anchor, factor);
}

pub fn hover_details(
    response: &egui::Response,
    image_rect: egui::Rect,
    series: &SpectrogramSeries,
    visible: &[crate::spectrogram::model::SpectrogramRow],
    row_start: usize,
    source_cols: &[usize],
    total_rows: usize,
) -> String {
    let Some(hover) = response.hover_pos() else {
        return String::new();
    };
    if !image_rect.contains(hover) || visible.is_empty() || source_cols.is_empty() {
        return String::new();
    }
    let rel_x = ((hover.x - image_rect.left()) / image_rect.width().max(1.0)).clamp(0.0, 1.0);
    let rel_y = ((hover.y - image_rect.top()) / image_rect.height().max(1.0)).clamp(0.0, 1.0);
    let col = ((rel_x * source_cols.len() as f32) as usize).min(source_cols.len() - 1);
    let row_index = ((rel_y * visible.len() as f32) as usize).min(visible.len() - 1);
    let source_col = source_cols[col];
    let row = &visible[row_index];
    let absolute = row_start + row_index;
    spectrogram_hover_text(&SpectrogramHoverInfo {
        energy_kev: series.energies_kev.get(source_col).copied().unwrap_or(0.0),
        counts: row.counts.get(source_col).copied().unwrap_or(0),
        channel: source_col,
        absolute_row: absolute,
        total_rows,
        age_secs: series.age_secs_before(absolute),
        interval_secs: row.interval_secs,
        row_total: row.row_total(),
        kind: row.kind,
    })
}
