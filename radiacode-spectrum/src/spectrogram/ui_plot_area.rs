use egui::{Color32, Context, Image, Rect, Response, RichText, Sense, Ui};
use radiacode_nuclides::SourceSummary;

use crate::app_config::AppConfig;
use crate::identify::PeakAnalysis;
use crate::layout::safe_span;
use crate::peak_overlay::{
    CLASSIFICATION_STRIP_HEIGHT, SpectrumPlotAction, draw_classification_strip,
};
use crate::spectrogram::axes::{count_rate_axis_label, x_axis_label, y_axis_label};
use crate::spectrogram::layout::compute_layout;
use crate::spectrogram::overlays::draw_grid;
use crate::spectrogram::peak_analysis::{
    channels_for_view, peak_analysis_for_view, series_for_peak_data,
};
use crate::spectrogram::preview::{
    channel_totals, draw_preview_controls, split_preview_area, strip_rect,
};
use crate::spectrogram::state::SpectrogramState;
use crate::spectrogram::texture::source_columns;
use crate::spectrogram::texture_sync::sync_texture;
use crate::spectrogram::ui_plot_layers::{SeriesLayers, draw_series_layers};
use crate::spectrogram::view_interact::handle_view_interaction;
use crate::theme::MUTED;

pub const AXIS_LEFT: f32 = 56.0;
pub const AXIS_RIGHT: f32 = 56.0;
pub const AXIS_BOTTOM: f32 = 40.0;
pub const PLOT_MIN: f32 = 80.0;
const PLOT_BACKGROUND: Color32 = Color32::from_rgb(8, 10, 16);

pub fn draw_spectrogram_plot_area(
    ui: &mut Ui,
    ctx: &Context,
    state: &mut SpectrogramState,
    config: &AppConfig,
) -> Option<SpectrumPlotAction> {
    if state.active_series().is_none() {
        ui.label(
            RichText::new("Empty spectrogram grid. Connect a device to start capturing.")
                .small()
                .color(MUTED),
        );
    }
    let plot_size = plot_size(ui.available_size(), state.show_peaks);
    let peak_analysis = peak_analysis_for_view(state, config);
    ui.horizontal(|ui| {
        draw_axis_gutter(ui, plot_size.y, AXIS_LEFT, |ui| {
            draw_preview_controls(ui, state);
            ui.add_space(8.0);
            muted_label(ui, y_axis_label());
        });
        let (rect, response) = ui.allocate_exact_size(plot_size, Sense::click_and_drag());
        draw_axis_gutter(ui, plot_size.y, AXIS_RIGHT, |ui| {
            muted_label(ui, count_rate_axis_label());
        });
        draw_plot(ui, ctx, state, rect, &response, peak_analysis.as_ref());
    });
    ui.label(RichText::new(x_axis_label()).small().color(MUTED));
    if !state.show_peaks {
        return None;
    }
    draw_peak_summary(ui, peak_analysis.as_ref())
}

fn plot_size(available: egui::Vec2, show_peaks: bool) -> egui::Vec2 {
    let strip_h = if show_peaks {
        CLASSIFICATION_STRIP_HEIGHT
    } else {
        0.0
    };
    egui::vec2(
        safe_span(available.x, AXIS_LEFT + AXIS_RIGHT, PLOT_MIN),
        safe_span(available.y, AXIS_BOTTOM + strip_h, PLOT_MIN),
    )
}

fn draw_axis_gutter(ui: &mut Ui, height: f32, width: f32, content: impl FnOnce(&mut Ui)) {
    ui.allocate_ui_with_layout(
        egui::vec2(safe_span(width, 4.0, 40.0), height),
        egui::Layout::top_down(egui::Align::Min),
        content,
    );
}

fn muted_label(ui: &mut Ui, text: &str) {
    ui.with_layout(egui::Layout::top_down(egui::Align::Max), |ui| {
        ui.label(RichText::new(text).small().color(MUTED));
    });
}

fn draw_plot(
    ui: &mut Ui,
    ctx: &Context,
    state: &mut SpectrogramState,
    rect: Rect,
    response: &Response,
    peak_analysis: Option<&PeakAnalysis>,
) {
    let total_rows = state
        .active_series()
        .map(|series| series.row_count())
        .unwrap_or(0);
    let (preview_rect, grid_rect) = split_preview_area(rect);
    let layout_before = compute_layout(
        grid_rect,
        channels_for_view(state),
        state.view_range.fit_full_spectrum,
    );
    handle_view_interaction(
        ui,
        response,
        grid_rect,
        layout_before.image_rect,
        layout_before,
        state,
        total_rows,
    );
    let layout = compute_layout(
        grid_rect,
        channels_for_view(state),
        state.view_range.fit_full_spectrum,
    );
    state
        .view_range
        .clamp_to_history(total_rows, layout.display_rows);
    sync_texture(ctx, state, layout);
    let texture_id = state
        .texture_handle
        .as_ref()
        .map(|texture| texture.id())
        .expect("texture handle");
    ui.painter_at(rect).rect_filled(rect, 0.0, PLOT_BACKGROUND);
    Image::new((texture_id, layout.image_rect.size())).paint_at(ui, layout.image_rect);
    let painter = ui.painter().clone();
    draw_grid(&painter, layout.image_rect, layout, true);
    let totals = series_for_peak_data(state).map(|active| {
        let token = crate::peaks::spectrogram_series_peak_token(&active);
        state
            .totals_memo
            .get_or_compute(token, || channel_totals(&active))
    });
    let row_start = state
        .view_range
        .visible_start(total_rows, layout.display_rows);
    let Some(series) = state.active_series() else {
        return;
    };
    let source_cols = source_columns(
        series,
        state.view_range.energy_min_kev,
        state.view_range.energy_max_kev,
        state.view_range.channel_start,
        layout.display_cols,
    );
    let layers = SeriesLayers {
        series,
        source_cols: &source_cols,
        visible: series.row_window(row_start, layout.display_rows),
        layout,
        preview_area: strip_rect(preview_rect, layout.image_rect),
        totals: totals.as_deref().map(Vec::as_slice),
        peak_analysis,
        preview_scale: state.preview_scale,
        display: state.display,
        row_start,
        total_rows,
        follow_live: state.view_range.follow_live,
    };
    draw_series_layers(ui, response, &painter, layers);
}

fn draw_peak_summary(
    ui: &mut Ui,
    peak_analysis: Option<&PeakAnalysis>,
) -> Option<SpectrumPlotAction> {
    let peak_count = peak_analysis
        .map(|analysis| analysis.peaks.len())
        .unwrap_or(0);
    let default_sources = SourceSummary::default();
    let sources = peak_analysis
        .map(|analysis| &analysis.sources)
        .unwrap_or(&default_sources);
    draw_classification_strip(ui, peak_count, sources)
}
