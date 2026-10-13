use radiacode_nuclides::{NuclideId, nuclide_by_id, source_for_nuclide, strongest_gamma};

use super::SpectrumApp;
use crate::peak_id::PeakIdAction;
use crate::peak_overlay::SpectrumPlotAction;
use crate::sources_tab::SourcesAction;
use crate::view_tab::ViewTab;

impl SpectrumApp {
    pub(super) fn handle_spectrum_plot_action(&mut self, action: SpectrumPlotAction) {
        match action {
            SpectrumPlotAction::ShowSourceFor(id) => self.show_source_for(id),
        }
    }

    pub(super) fn handle_peak_id_action(&mut self, action: PeakIdAction) {
        match action {
            PeakIdAction::OpenSource(key) => self.open_source(key),
        }
    }

    pub(super) fn handle_sources_action(&mut self, action: SourcesAction) {
        match action {
            SourcesAction::FindEnergy(energy_kev) => self.open_peak_id(Some(energy_kev)),
        }
    }

    fn show_source_for(&mut self, id: NuclideId) {
        match source_for_nuclide(id) {
            Some(source) => self.open_source(source.key),
            None => self.open_peak_id(strongest_gamma_energy(id)),
        }
    }

    fn open_source(&mut self, key: &str) {
        self.sources.reveal(key);
        self.tabs.active = ViewTab::Sources;
    }

    fn open_peak_id(&mut self, energy_kev: Option<f64>) {
        if let Some(energy_kev) = energy_kev {
            self.peak_id.set_energy(energy_kev);
        }
        self.tabs.active = ViewTab::PeakId;
    }
}

fn strongest_gamma_energy(id: NuclideId) -> Option<f64> {
    nuclide_by_id(id)
        .and_then(|nuclide| strongest_gamma(&nuclide.gammas))
        .map(|line| line.energy_kev)
}
