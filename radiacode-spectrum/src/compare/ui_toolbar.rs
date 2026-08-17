use egui::Ui;

use crate::compare::state::CompareState;
use crate::layout::draw_toolbar;
use crate::plot_style::draw_plot_style_toggle;
use crate::scale::YScale;
use crate::theme::{ERROR, MUTED};
use crate::ui::widgets::draw_smoothing_slider;

pub fn draw_compare_toolbar(ui: &mut Ui, state: &mut CompareState, y_scale: &mut YScale) {
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
        ui.label(egui::RichText::new(&state.error).small().color(ERROR));
    }
}
