use std::sync::Arc;

use egui::{Color32, Context, Image, RichText, Sense, Ui};

use crate::app_config::AppConfig;
use crate::identify::{PeakAnalysis, analyze_peaks};
use crate::layout::safe_span;
use crate::peak_overlay::{SpectrumPlotAction, draw_source_chips, draw_spectrogram_peaks};
use crate::peaks::{DetectionParams, peaks_from_channel_totals, spectrogram_series_peak_token};
use crate::spectrogram::axes::{draw_x_axis, x_axis_label, y_axis_label};
use crate::spectrogram::count_rate::draw_count_rate_overlay;
use crate::spectrogram::layout::{
    DEFAULT_EMPTY_CHANNELS, channels_in_energy_range, compute_layout,
};
use crate::spectrogram::model::SpectrogramDisplay;
use crate::spectrogram::overlays::{draw_crosshair, draw_grid};
use crate::spectrogram::preview::{
    channel_totals, draw_preview_controls, draw_preview_strip, preview_strip_response,
    split_preview_area, strip_rect,
};
use crate::spectrogram::state::SpectrogramState;
use crate::spectrogram::texture::source_columns;
use crate::spectrogram::texture_sync::sync_texture;
use crate::spectrogram::time_axis::draw_time_axis;
use crate::spectrogram::view_interact::{handle_view_interaction, hover_details};
use crate::theme::MUTED;

pub const AXIS_LEFT: f32 = 56.0;
pub const AXIS_BOTTOM: f32 = 40.0;
pub const PLOT_MIN: f32 = 80.0;

pub fn draw_spectrogram_plot_area(
    ui: &mut Ui,
    ctx: &Context,
    state: &mut SpectrogramState,
    config: &AppConfig,
) -> Option<SpectrumPlotAction> {
    let total_rows = state
        .active_series()
        .map(|series| series.row_count())
        .unwrap_or(0);
    if state.active_series().is_none() {
        ui.label(
            RichText::new("Empty spectrogram grid. Connect a device to start capturing.")
                .small()
                .color(MUTED),
        );
    }
    let x_label = x_axis_label();
    let available = ui.available_size();
    let plot_size = egui::vec2(
        safe_span(available.x, AXIS_LEFT, PLOT_MIN),
        safe_span(available.y, AXIS_BOTTOM, PLOT_MIN),
    );
    let peak_analysis = peak_analysis_for_view(state, config);
    let mut peak_sources = None;

    ui.horizontal(|ui| {
        ui.allocate_ui_with_layout(
            egui::vec2(safe_span(AXIS_LEFT, 4.0, 40.0), plot_size.y),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                draw_preview_controls(ui, state);
                ui.add_space(8.0);
                ui.with_layout(egui::Layout::top_down(egui::Align::Max), |ui| {
                    ui.label(RichText::new(y_axis_label()).small().color(MUTED));
                });
            },
        );
        let (rect, response) = ui.allocate_exact_size(plot_size, Sense::click_and_drag());
        let (preview_rect, grid_rect) = split_preview_area(rect);
        let channels_before = channels_for_view(state);
        let layout_before = compute_layout(
            grid_rect,
            channels_before,
            state.view_range.fit_full_spectrum,
        );
        handle_view_interaction(
            ui,
            &response,
            grid_rect,
            layout_before.image_rect,
            layout_before,
            state,
            total_rows,
        );

        let energy_min = state.view_range.energy_min_kev;
        let energy_max = state.view_range.energy_max_kev;
        let channels = channels_for_view(state);
        let layout = compute_layout(grid_rect, channels, state.view_range.fit_full_spectrum);
        state
            .view_range
            .clamp_to_history(total_rows, layout.display_rows);
        sync_texture(ctx, state, layout);

        let texture_id = state
            .texture_handle
            .as_ref()
            .map(|texture| texture.id())
            .expect("texture handle");
        let plot_painter = ui.painter_at(rect);
        plot_painter.rect_filled(rect, 0.0, Color32::from_rgb(8, 10, 16));
        Image::new((
            texture_id,
            egui::vec2(layout.image_rect.width(), layout.image_rect.height()),
        ))
        .paint_at(ui, layout.image_rect);

        let row_start = state
            .view_range
            .visible_start(total_rows, layout.display_rows);
        let axis_painter = ui.painter().clone();
        draw_grid(&axis_painter, layout.image_rect, layout, state.show_grid);
        let series_for_totals = series_for_peak_data(state);
        let totals = series_for_totals.as_ref().map(|active| {
            let token = spectrogram_series_peak_token(active);
            state
                .totals_memo
                .get_or_compute(token, || channel_totals(active))
        });
        if let Some(series) = state.active_series() {
            let source_cols = source_columns(
                series,
                energy_min,
                energy_max,
                state.view_range.channel_start,
                layout.display_cols,
            );
            let visible = series.row_window(row_start, layout.display_rows);
            let preview_area = strip_rect(preview_rect, layout.image_rect);
            if let Some(totals) = totals {
                draw_preview_strip(
                    &axis_painter,
                    preview_area,
                    series,
                    &source_cols,
                    totals.as_ref(),
                    state.preview_scale,
                    peak_analysis.as_ref(),
                );
            }
            preview_strip_response(
                &response,
                preview_area,
                series,
                state.display,
                total_rows,
                state.view_range.follow_live,
            );
            if let Some(analysis) = peak_analysis.as_ref() {
                draw_spectrogram_peaks(
                    &axis_painter,
                    layout.image_rect,
                    series,
                    &source_cols,
                    &analysis.identifications,
                );
                peak_sources = Some(analysis.sources.clone());
            }
            draw_count_rate_overlay(
                &axis_painter,
                layout.image_rect,
                layout,
                visible,
                state.show_count_rate,
            );
            if let Some(hover) = response.hover_pos() {
                draw_crosshair(&axis_painter, hover, layout.image_rect, Some(preview_area));
            }
            draw_time_axis(&axis_painter, layout.image_rect, layout, visible);
            draw_x_axis(&axis_painter, ui, layout.image_rect, series, &source_cols);
            let details = hover_details(
                &response,
                layout.image_rect,
                series,
                visible,
                row_start,
                &source_cols,
                total_rows,
            );
            if !details.is_empty() {
                response.clone().on_hover_text(details);
            }
        }
    });
    ui.label(RichText::new(x_label).small().color(MUTED));
    peak_sources.and_then(|sources| draw_source_chips(ui, &sources))
}

fn peak_analysis_for_view(
    state: &mut SpectrogramState,
    config: &AppConfig,
) -> Option<PeakAnalysis> {
    if !state.show_peaks {
        return None;
    }
    let series_for_peaks = series_for_peak_data(state)?;
    let params = DetectionParams::from_app_config(config);
    let token = spectrogram_series_peak_token(&series_for_peaks);
    let totals = state
        .totals_memo
        .get_or_compute(token, || channel_totals(&series_for_peaks));
    let channel_count = series_for_peaks.header.channel_count as usize;
    let energies: Vec<f64> = series_for_peaks
        .energies_kev
        .iter()
        .take(channel_count)
        .copied()
        .collect();
    let key = crate::peaks::PeakMemoKey::new(token, params);
    let peaks = state
        .peak_memo
        .get_or_compute(key, || {
            peaks_from_channel_totals(&energies, totals.as_ref(), params)
        })
        .to_vec();
    Some(analyze_peaks(&peaks, config))
}

fn series_for_peak_data(
    state: &SpectrogramState,
) -> Option<Arc<crate::spectrogram::model::SpectrogramSeries>> {
    match state.display {
        SpectrogramDisplay::Live => state.live_series.clone(),
        SpectrogramDisplay::Loaded => state
            .loaded_series
            .as_ref()
            .map(|series| Arc::new(series.clone())),
    }
}

fn channels_for_view(state: &SpectrogramState) -> usize {
    state
        .active_series()
        .map(|series| {
            channels_in_energy_range(
                &series.energies_kev,
                state.view_range.energy_min_kev,
                state.view_range.energy_max_kev,
            )
            .max(1)
        })
        .unwrap_or(DEFAULT_EMPTY_CHANNELS)
}
