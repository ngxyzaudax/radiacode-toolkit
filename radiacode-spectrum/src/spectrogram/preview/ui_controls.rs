use egui::{RichText, Ui, Vec2};

use crate::scale::YScale;
use crate::spectrogram::state::SpectrogramState;
use crate::theme::{ACCENT, MUTED};

const CONTROL_SIZE: Vec2 = Vec2::new(48.0, 18.0);

pub fn draw_preview_controls(ui: &mut Ui, state: &mut SpectrogramState) {
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 4.0;
        let scale_label = match state.preview_scale {
            YScale::Linear => "Lin",
            YScale::Logarithmic => "Log",
        };
        if ui
            .add(egui::Button::new(RichText::new(scale_label).small()).min_size(CONTROL_SIZE))
            .on_hover_text("Toggle linear / logarithmic scale")
            .clicked()
        {
            state.preview_scale = match state.preview_scale {
                YScale::Linear => YScale::Logarithmic,
                YScale::Logarithmic => YScale::Linear,
            };
        }
        let peaks_active = state.show_peaks;
        let peaks_button = ui.add(
            egui::Button::new(RichText::new("Peaks").small().color(if peaks_active {
                ACCENT
            } else {
                MUTED
            }))
            .min_size(CONTROL_SIZE)
            .fill(if peaks_active {
                egui::Color32::from_rgba_unmultiplied(72, 132, 196, 48)
            } else {
                egui::Color32::TRANSPARENT
            }),
        );
        if peaks_button
            .on_hover_text("Toggle peak detection overlay")
            .clicked()
        {
            state.show_peaks = !state.show_peaks;
        }
    });
}
