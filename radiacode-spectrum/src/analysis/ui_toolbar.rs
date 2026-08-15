use egui::{RichText, Ui};

use crate::analysis::state::AnalysisState;
use crate::layout::draw_toolbar;
use crate::scale::YScale;
use crate::smooth::normalize_window;
use crate::theme::MUTED;

pub fn draw_analysis_toolbar(ui: &mut Ui, state: &mut AnalysisState, y_scale: &mut YScale) {
    draw_toolbar(ui, |ui| {
        ui.selectable_value(y_scale, YScale::Linear, "Linear");
        ui.selectable_value(y_scale, YScale::Logarithmic, "Log");
        ui.selectable_value(&mut state.outline_only, false, "Filled");
        ui.selectable_value(&mut state.outline_only, true, "Outline");
        ui.add_enabled_ui(state.background.is_some(), |ui| {
            ui.checkbox(&mut state.subtract_background, "Subtract BG");
        });
        ui.checkbox(&mut state.show_peaks, "Peaks");
        ui.add_enabled_ui(state.show_peaks, |ui| {
            ui.checkbox(&mut state.identify_isotopes, "Identify");
        });
        ui.label("Smoothing");
        let mut slider = state.smooth_window.clamp(1, 16) as i32;
        if ui
            .add(egui::Slider::new(&mut slider, 1..=16).text("channels"))
            .changed()
        {
            state.smooth_window = normalize_window(slider as usize);
        }
    });
    if !state.status.is_empty() {
        ui.label(RichText::new(&state.status).small().color(MUTED));
    }
    if !state.error.is_empty() {
        ui.label(
            RichText::new(&state.error)
                .small()
                .color(egui::Color32::from_rgb(220, 120, 120)),
        );
    }
}
