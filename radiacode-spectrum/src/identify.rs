use radiacode_nuclides::{
    MatchParams, PeakIdentification, SpectrumPeak as NuclidePeak, match_peaks,
};

use crate::app_config::AppConfig;
use crate::peak_detect::SpectrumPeak;

pub fn match_params_from_config(config: &AppConfig) -> MatchParams {
    MatchParams {
        relative_frac: config.match_tolerance_frac,
        floor_kev: config.match_tolerance_floor_kev,
        min_intensity_pct: config.match_min_intensity_pct,
    }
}

pub fn identify_peaks(
    peaks: &[SpectrumPeak],
    config: &AppConfig,
) -> Vec<PeakIdentification> {
    let params = match_params_from_config(config);
    let converted = peaks
        .iter()
        .map(|peak| NuclidePeak {
            energy_kev: peak.energy_kev,
            counts: peak.counts,
        })
        .collect::<Vec<_>>();
    match_peaks(&converted, params)
}
