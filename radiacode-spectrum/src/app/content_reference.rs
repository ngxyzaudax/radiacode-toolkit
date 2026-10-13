use egui::Ui;

use super::SpectrumApp;
use crate::peak_id::{PeakIdProps, draw_peak_id_view};
use crate::peaks::{DetectionParams, PeakMemoKey, peaks_from_spectrum_view};
use crate::sources_tab::{SourcesViewProps, draw_sources_view};

impl SpectrumApp {
    pub(super) fn draw_peak_id_tab(&mut self, ui: &mut Ui) {
        let peaks_kev = self.current_peak_energies();
        let props = PeakIdProps {
            fwhm_pct: self.settings.app.detector_fwhm_pct,
            min_supported_half_life_secs: self.settings.app.match_min_supported_half_life_secs,
            peaks_kev: &peaks_kev,
        };
        if let Some(action) = draw_peak_id_view(ui, &mut self.peak_id, props) {
            self.handle_peak_id_action(action);
        }
    }

    pub(super) fn draw_sources_tab(&mut self, ui: &mut Ui) {
        let props = SourcesViewProps {
            fwhm_pct: self.settings.app.detector_fwhm_pct,
            measured: self.state.spectrum.as_ref(),
        };
        if let Some(action) = draw_sources_view(ui, &mut self.sources, props) {
            self.handle_sources_action(action);
        }
    }

    pub(super) fn current_peak_energies(&mut self) -> Vec<f64> {
        let Some(spectrum) = self.state.spectrum.as_ref() else {
            return Vec::new();
        };
        let params = DetectionParams::from_app_config(&self.settings.app);
        let key = PeakMemoKey::new(self.state.spectrum_sequence, params);
        let mut energies: Vec<f64> = self
            .view
            .peak_memo
            .get_or_compute(key, || peaks_from_spectrum_view(spectrum, params))
            .iter()
            .map(|peak| peak.energy_kev)
            .collect();
        energies.sort_by(f64::total_cmp);
        energies
    }
}
