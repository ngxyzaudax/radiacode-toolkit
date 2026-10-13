use egui::{Color32, Context, Image, Rect, Response, Ui};

use crate::identify::PeakAnalysis;
use crate::spectrogram::layout::compute_layout;
use crate::spectrogram::overlays::draw_grid;
use crate::spectrogram::peak_analysis::{channels_for_view, series_for_peak_data};
use crate::spectrogram::preview::{channel_totals, split_preview_area, strip_rect};
use crate::spectrogram::state::SpectrogramState;
use crate::spectrogram::texture::source_columns;
use crate::spectrogram::texture_sync::sync_texture;
use crate::spectrogram::ui_plot_layers::{SeriesLayers, draw_series_layers};
use crate::spectrogram::view_interact::handle_view_interaction;

const PLOT_BACKGROUND: Color32 = Color32::from_rgb(8, 10, 16);

pub fn draw_plot(
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
    draw_series_layers(response, &painter, layers);
}
