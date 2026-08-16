use radiacode_nuclides::{
    MatchParams, PeakIdentification, SourceSummary, match_peaks, summarize_sources,
    SpectrumPeak as NuclidePeak,
};

use crate::app_config::AppConfig;
use crate::peaks::{DetectedPeak, DetectionParams};

pub fn detection_params_from_config(config: &AppConfig) -> DetectionParams {
    DetectionParams::from_app_config(config)
}

pub fn match_params_from_config(config: &AppConfig) -> MatchParams {
    MatchParams {
        relative_frac: config.match_tolerance_frac,
        floor_kev: config.match_tolerance_floor_kev,
        min_intensity_pct: config.match_min_intensity_pct,
        detector_fwhm_pct: config.detector_fwhm_pct,
        tolerance_fwhm_frac: 0.5,
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeakAnalysis {
    pub peaks: Vec<DetectedPeak>,
    pub identifications: Vec<PeakIdentification>,
    pub sources: SourceSummary,
}

pub fn analyze_peaks(peaks: &[DetectedPeak], config: &AppConfig) -> PeakAnalysis {
    let params = match_params_from_config(config);
    let converted = peaks
        .iter()
        .map(|peak| NuclidePeak {
            energy_kev: peak.energy_kev,
            counts: peak.net_area,
        })
        .collect::<Vec<_>>();
    let identifications = match_peaks(&converted, params);
    let sources = summarize_sources(&identifications);
    PeakAnalysis {
        peaks: peaks.to_vec(),
        identifications,
        sources,
    }
}
