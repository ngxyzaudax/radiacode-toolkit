use egui::Ui;

use crate::analysis::state::AnalysisState;
use crate::layout::draw_toolbar;
use crate::plot_style::draw_plot_style_toggle;
use crate::scale::YScale;
use crate::theme::MUTED;
use crate::ui::widgets::draw_smoothing_slider;

pub fn draw_analysis_toolbar(ui: &mut Ui, state: &mut AnalysisState, y_scale: &mut YScale) {
    draw_toolbar(ui, |ui| {
        ui.selectable_value(y_scale, YScale::Linear, "Linear");
        ui.selectable_value(y_scale, YScale::Logarithmic, "Log");
        draw_plot_style_toggle(ui, &mut state.outline_only);
        ui.add_enabled_ui(state.background.is_some(), |ui| {
            ui.checkbox(&mut state.subtract_background, "Subtract BG");
        });
        ui.checkbox(&mut state.show_peaks, "Peak detection");
        draw_smoothing_slider(ui, "Smoothing", &mut state.smooth_window, Some("channels"));
    });
    if !state.status.is_empty() {
        ui.label(egui::RichText::new(&state.status).small().color(MUTED));
    }
    if !state.error.is_empty() {
        ui.label(
            egui::RichText::new(&state.error)
                .small()
                .color(egui::Color32::from_rgb(220, 120, 120)),
        );
    }
}
