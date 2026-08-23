use egui::Ui;
use radiacode_nuclides::SourceSummary;

use crate::layout::safe_span;
use crate::peak_overlay::action::SpectrumPlotAction;
use crate::peak_overlay::classification_panel::{
    CLASSIFICATION_STRIP_HEIGHT, draw_classification_strip,
};

const PLOT_MIN: f32 = 80.0;

pub fn draw_with_classification_strip(
    ui: &mut Ui,
    show_peaks: bool,
    peak_count: usize,
    sources: Option<&SourceSummary>,
    draw_plot: impl FnOnce(&mut Ui, f32),
) -> Option<SpectrumPlotAction> {
    let strip_h = if show_peaks {
        CLASSIFICATION_STRIP_HEIGHT
    } else {
        0.0
    };
    let plot_h = safe_span(ui.available_height(), strip_h, PLOT_MIN);
    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), plot_h),
        egui::Layout::top_down(egui::Align::LEFT),
        |ui| {
            ui.set_min_height(plot_h);
            draw_plot(ui, plot_h);
        },
    );
    if !show_peaks {
        return None;
    }
    let sources = sources.cloned().unwrap_or_default();
    draw_classification_strip(ui, peak_count, &sources)
}
