use egui::Ui;
use radiacode_nuclides::SourceSummary;

use crate::peak_overlay::action::SpectrumPlotAction;
use crate::theme::MUTED;

pub fn draw_source_chips(ui: &mut Ui, sources: &SourceSummary) -> Option<SpectrumPlotAction> {
    if sources.chains.is_empty() && sources.nuclides.is_empty() {
        return None;
    }
    let mut action = None;
    ui.horizontal_wrapped(|ui| {
        for chain in &sources.chains {
            let label = format!("{} - {} members", chain.name, chain.matched_members);
            if ui.button(label).clicked() {
                action = Some(SpectrumPlotAction::OpenCatalogueChain(chain.head));
            }
        }
        for nuclide in &sources.nuclides {
            let label = format!(
                "{} ({} line{})",
                nuclide.display_name,
                nuclide.matched_lines,
                if nuclide.matched_lines == 1 { "" } else { "s" }
            );
            if ui.button(label).clicked() {
                action = Some(SpectrumPlotAction::OpenCatalogue(nuclide.id));
            }
        }
    });
    ui.add_space(4.0);
    ui.label(
        egui::RichText::new("Click a source to open it in the catalogue.")
            .small()
            .color(MUTED),
    );
    action
}
