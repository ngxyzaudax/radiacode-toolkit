use egui::{Color32, RichText, Ui};

use crate::compare::state::ComparedSample;
use crate::theme::{COMPARE_BACKGROUND, compare_sample_color};

pub fn draw_legend(ui: &mut Ui, samples: &[ComparedSample], has_background: bool) {
    ui.horizontal_wrapped(|ui| {
        if has_background {
            legend_swatch(ui, "Background", COMPARE_BACKGROUND);
        }
        for (index, sample) in samples.iter().enumerate() {
            legend_swatch(ui, &sample.spectrum.name, compare_sample_color(index));
        }
    });
    ui.add_space(6.0);
}

fn legend_swatch(ui: &mut Ui, label: &str, color: Color32) {
    egui::Frame::new()
        .fill(color.gamma_multiply(0.22))
        .stroke(egui::Stroke::new(1.0, color))
        .corner_radius(3)
        .inner_margin(egui::Margin::symmetric(6, 2))
        .show(ui, |ui| {
            ui.label(RichText::new(label).small().strong().color(color));
        });
}
