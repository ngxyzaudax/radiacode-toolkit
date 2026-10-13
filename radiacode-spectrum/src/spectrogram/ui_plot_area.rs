use egui::{Context, RichText, Sense, Ui};
use radiacode_nuclides::SourceSummary;

use crate::app_config::AppConfig;
use crate::identify::PeakAnalysis;
use crate::layout::safe_span;
use crate::peak_overlay::{
    CLASSIFICATION_STRIP_HEIGHT, SpectrumPlotAction, draw_classification_strip,
};
use crate::spectrogram::axes::{count_rate_axis_label, x_axis_label, y_axis_label};
use crate::spectrogram::peak_analysis::peak_analysis_for_view;
use crate::spectrogram::preview::draw_preview_controls;
use crate::spectrogram::state::SpectrogramState;
use crate::spectrogram::ui_plot_grid::draw_plot;
use crate::theme::MUTED;

pub const AXIS_LEFT: f32 = 56.0;
pub const AXIS_RIGHT: f32 = 56.0;
pub const AXIS_BOTTOM: f32 = 40.0;
pub const PLOT_MIN: f32 = 80.0;

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
