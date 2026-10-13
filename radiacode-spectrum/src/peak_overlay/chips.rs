use egui::Ui;
use radiacode_nuclides::SourceSummary;

use crate::peak_overlay::action::SpectrumPlotAction;

const MAX_CHAINS: usize = 4;
const MAX_NUCLIDES: usize = 6;
const CHAIN_SCORE_FLOOR_FRAC: f64 = 0.2;

pub fn draw_ranked_chips(ui: &mut Ui, sources: &SourceSummary) -> Option<SpectrumPlotAction> {
    let chains = ranked_chains(sources);
    let nuclides = ranked_nuclides(sources);
    if chains.is_empty() && nuclides.is_empty() {
        return None;
    }
    let mut action = None;
    ui.horizontal_wrapped(|ui| {
        for chain in chains {
            let label = format!("{} - {} members", chain.name, chain.matched_members);
            if ui.button(label).clicked() {
                action = Some(SpectrumPlotAction::OpenCatalogueChain(chain.head));
            }
        }
        for nuclide in nuclides {
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
    action
}

fn ranked_chains(sources: &SourceSummary) -> Vec<&radiacode_nuclides::ChainEvidence> {
    let top_score = sources
        .chains
        .first()
        .map(|chain| chain.score)
        .unwrap_or(0.0);
    let floor = top_score * CHAIN_SCORE_FLOOR_FRAC;
    sources
        .chains
        .iter()
        .filter(|chain| chain.score >= floor && chain.matched_members >= 1)
        .take(MAX_CHAINS)
        .collect()
}

fn ranked_nuclides(sources: &SourceSummary) -> Vec<&radiacode_nuclides::NuclideEvidence> {
    sources.nuclides.iter().take(MAX_NUCLIDES).collect()
}
