use radiacode_nuclides::{
    MatchParams, PeakIdentification, SourceSummary, SpectrumPeak as NuclidePeak,
    analyze_peaks as nuclide_analyze_peaks,
};

use crate::app_config::AppConfig;
use crate::peaks::DetectedPeak;

pub use radiacode_nuclides::AnalysisWindow;

pub fn match_params_from_config(config: &AppConfig) -> MatchParams {
    MatchParams {
        relative_frac: config.match_tolerance_frac,
        floor_kev: config.match_tolerance_floor_kev,
        min_intensity_pct: config.match_min_intensity_pct,
        detector_fwhm_pct: config.detector_fwhm_pct,
        tolerance_fwhm_frac: 0.5,
        min_supported_half_life_secs: config.match_min_supported_half_life_secs,
        absence_penalty: 0.15,
        low_energy_cutoff_kev: 100.0,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeakAnalysis {
    pub peaks: Vec<DetectedPeak>,
    pub identifications: Vec<PeakIdentification>,
    pub sources: SourceSummary,
}

pub fn analyze_peaks(
    peaks: &[DetectedPeak],
    config: &AppConfig,
    window: AnalysisWindow,
) -> PeakAnalysis {
    let params = match_params_from_config(config);
    let converted = peaks
        .iter()
        .map(|peak| NuclidePeak {
            energy_kev: peak.energy_kev,
            counts: peak.net_area,
        })
        .collect::<Vec<_>>();
    let result = nuclide_analyze_peaks(&converted, params, window);
    PeakAnalysis {
        peaks: peaks.to_vec(),
        identifications: result.identifications,
        sources: result.sources,
    }
}

pub fn analysis_window_from_energies(energies_kev: &[f64]) -> AnalysisWindow {
    if energies_kev.is_empty() {
        return AnalysisWindow::default();
    }
    let min_kev = energies_kev.iter().copied().fold(f64::INFINITY, f64::min);
    let max_kev = energies_kev.iter().copied().fold(0.0_f64, f64::max);
    AnalysisWindow {
        min_kev: min_kev.min(max_kev),
        max_kev,
    }
}
