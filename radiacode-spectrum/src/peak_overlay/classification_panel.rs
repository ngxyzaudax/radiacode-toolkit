use egui::{RichText, Ui};
use radiacode_nuclides::SourceSummary;

use crate::peak_overlay::action::SpectrumPlotAction;
use crate::peak_overlay::chips::draw_ranked_chips;
use crate::peak_overlay::verdict::draw_verdict;
use crate::theme::MUTED;

pub const CLASSIFICATION_STRIP_HEIGHT: f32 = 88.0;

pub fn draw_classification_strip(
    ui: &mut Ui,
    peak_count: usize,
    sources: &SourceSummary,
) -> Option<SpectrumPlotAction> {
    let width = ui.available_width();
    ui.allocate_ui_with_layout(
        egui::vec2(width, CLASSIFICATION_STRIP_HEIGHT),
        egui::Layout::top_down(egui::Align::LEFT),
        |ui| {
            ui.set_min_height(CLASSIFICATION_STRIP_HEIGHT);
            egui::Frame::new()
                .inner_margin(egui::Margin::symmetric(8, 4))
                .show(ui, |ui| {
                    ui.separator();
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format!("{peak_count} detected peaks"))
                                .small()
                                .color(MUTED),
                        );
                        ui.separator();
                        draw_verdict(ui, sources);
                    });
                    ui.add_space(4.0);
                    let action = draw_ranked_chips(ui, sources);
                    ui.label(
                        RichText::new("Click a source to open it in the catalogue.")
                            .small()
                            .color(MUTED),
                    );
                    action
                })
                .inner
        },
    )
    .inner
}
