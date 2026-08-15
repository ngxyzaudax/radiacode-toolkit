use egui::{Color32, Context, Image, RichText, Sense, Ui};

use crate::app_config::AppConfig;
use crate::identify::identify_peaks;
use crate::layout::{draw_master_detail, safe_span, MasterDetailRegion};
use crate::model::ConnectionState;
use crate::peak_overlay::draw_identified_spectrogram_lines;
use crate::peak_profile::peaks_from_spectrogram_view;
use crate::spectrogram::axes::{draw_header_line, draw_x_axis, x_axis_label, y_axis_label};
use crate::spectrogram::controls_action::SpectrogramControlsAction;
use crate::spectrogram::count_rate::draw_count_rate_overlay;
use crate::spectrogram::layout::{
    DEFAULT_EMPTY_CHANNELS, channels_in_energy_range, compute_layout,
};
use crate::spectrogram::model::SpectrogramDisplay;
use crate::spectrogram::overlays::{draw_crosshair, draw_grid};
use crate::spectrogram::state::SpectrogramState;
use crate::spectrogram::texture::source_columns;
use crate::spectrogram::texture_sync::sync_texture;
use crate::spectrogram::time_axis::draw_time_axis;
use crate::spectrogram::ui_library::draw_library;
use crate::spectrogram::ui_toolbar::draw_spectrogram_toolbar;
use crate::spectrogram::view_interact::{handle_view_interaction, hover_details};
use crate::theme::MUTED;

const AXIS_LEFT: f32 = 56.0;
const AXIS_BOTTOM: f32 = 40.0;
const PLOT_MIN: f32 = 80.0;

pub fn draw_spectrogram_view(
    ui: &mut Ui,
    ctx: &Context,
    state: &mut SpectrogramState,
    config: &AppConfig,
    connection: ConnectionState,
) -> Option<SpectrogramControlsAction> {
    let mut action = draw_spectrogram_toolbar(ui, state, connection);
    let mut pane_open = state.pane_open;
    draw_master_detail(
        ui,
        "spectrogram_library",
        "Library",
        &mut pane_open,
        |ui, region| match region {
            MasterDetailRegion::Pane => draw_library(ui, state, &mut action),
            MasterDetailRegion::Detail => draw_spectrogram_plot(ui, ctx, state, config),
        },
    );
    state.pane_open = pane_open;
    action
}

fn draw_spectrogram_plot(
    ui: &mut Ui,
    ctx: &Context,
    state: &mut SpectrogramState,
    config: &AppConfig,
) {
    let total_rows = state
        .active_series()
        .map(|series| series.row_count())
        .unwrap_or(0);
    if let Some(series) = state.active_series() {
        draw_header_line(ui, series, state.display == SpectrogramDisplay::Loaded);
    } else {
        ui.label(
            RichText::new("Empty spectrogram grid. Connect a device to start capturing.")
                .small()
                .color(MUTED),
        );
    }
    let follow = if state.view_range.follow_live {
        "live"
    } else {
        "history"
    };
    ui.label(
        RichText::new(format!(
            "{} rows  |  {}  |  scroll zoom section, Shift+scroll history, drag pan, double-click fit all",
            total_rows, follow
        ))
        .small()
        .color(MUTED),
    );

    let y_label = y_axis_label();
    let x_label = x_axis_label();
    let available = ui.available_size();
    let plot_size = egui::vec2(
        safe_span(available.x, AXIS_LEFT, PLOT_MIN),
        safe_span(available.y, AXIS_BOTTOM, PLOT_MIN),
    );

    ui.horizontal(|ui| {
        if !y_label.is_empty() {
            ui.allocate_ui_with_layout(
                egui::vec2(safe_span(AXIS_LEFT, 4.0, 40.0), plot_size.y),
                egui::Layout::top_down(egui::Align::Max),
                |ui| {
                    ui.label(RichText::new(y_label).small().color(MUTED));
                },
            );
        }
        let (rect, response) = ui.allocate_exact_size(plot_size, Sense::click_and_drag());

        let channels_before = channels_for_view(state);
        let layout_before =
            compute_layout(rect, channels_before, state.view_range.fit_full_spectrum);
        handle_view_interaction(
            ui,
            &response,
            layout_before.image_rect,
            layout_before,
            state,
            total_rows,
        );

        let energy_min = state.view_range.energy_min_kev;
        let energy_max = state.view_range.energy_max_kev;
        let channels = channels_for_view(state);
        let layout = compute_layout(rect, channels, state.view_range.fit_full_spectrum);
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
        if let Some(series) = state.active_series() {
            let source_cols = source_columns(
                series,
                energy_min,
                energy_max,
                state.view_range.channel_start,
                layout.display_cols,
            );
            let visible = series.row_window(row_start, layout.display_rows);
            if state.show_isotopes {
                let peaks = peaks_from_spectrogram_view(series, visible, &source_cols, 3);
                let identifications = identify_peaks(&peaks, config);
                draw_identified_spectrogram_lines(
                    &axis_painter,
                    layout.image_rect,
                    energy_min,
                    energy_max,
                    &identifications,
                );
            } else if state.show_peaks {
                let peaks = peaks_from_spectrogram_view(series, visible, &source_cols, 3);
                crate::peak_overlay::draw_spectrogram_peaks(
                    &axis_painter,
                    layout.image_rect,
                    energy_min,
                    energy_max,
                    &peaks,
                );
            }
            draw_count_rate_overlay(
                &axis_painter,
                layout.image_rect,
                layout,
                visible,
                state.show_count_rate,
            );
            if let Some(hover) = response.hover_pos() {
                draw_crosshair(&axis_painter, hover, layout.image_rect);
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
