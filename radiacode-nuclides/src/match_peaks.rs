use crate::identify::candidates::{peak_candidates, rank_candidates};
use crate::model::NuclideId;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MatchParams {
    pub relative_frac: f64,
    pub floor_kev: f64,
    pub min_intensity_pct: f64,
    pub detector_fwhm_pct: f64,
    pub tolerance_fwhm_frac: f64,
    pub min_supported_half_life_secs: f64,
    pub absence_penalty: f64,
    pub low_energy_cutoff_kev: f64,
}

impl Default for MatchParams {
    fn default() -> Self {
        Self {
            relative_frac: 0.02,
            floor_kev: 3.0,
            min_intensity_pct: 1.7,
            detector_fwhm_pct: 7.0,
            tolerance_fwhm_frac: 0.5,
            min_supported_half_life_secs: 600.0,
            absence_penalty: 0.15,
            low_energy_cutoff_kev: 100.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpectrumPeak {
    pub energy_kev: f64,
    pub counts: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NuclideMatch {
    pub nuclide_id: NuclideId,
    pub display_name: String,
    pub line_energy_kev: f64,
    pub intensity_pct: f64,
    pub delta_kev: f64,
    pub score: f64,
    pub matched_lines: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeakIdentification {
    pub peak: SpectrumPeak,
    pub candidates: Vec<NuclideMatch>,
}

pub fn tolerance_kev(energy_kev: f64, params: MatchParams) -> f64 {
    let fwhm = resolution_fwhm_kev(energy_kev, params.detector_fwhm_pct);
    (params.tolerance_fwhm_frac * fwhm)
        .max(energy_kev * params.relative_frac)
        .max(params.floor_kev)
}

pub fn resolution_fwhm_kev(energy_kev: f64, fwhm_pct_at_662: f64) -> f64 {
    if energy_kev <= 0.0 || fwhm_pct_at_662 <= 0.0 {
        return 1.0;
    }
    const REFERENCE_ENERGY_KEV: f64 = 662.0;
    let reference_fwhm = REFERENCE_ENERGY_KEV * fwhm_pct_at_662 / 100.0;
    reference_fwhm * (energy_kev / REFERENCE_ENERGY_KEV).sqrt()
}

pub fn match_peaks(peaks: &[SpectrumPeak], params: MatchParams) -> Vec<PeakIdentification> {
    if peaks.is_empty() {
        return Vec::new();
    }
    let max_counts = peaks.iter().map(|peak| peak.counts).fold(1.0_f64, f64::max);
    peaks
        .iter()
        .map(|peak| {
            let normalized = SpectrumPeak {
                energy_kev: peak.energy_kev,
                counts: peak.counts / max_counts,
            };
            let candidates = rank_candidates(peak_candidates(normalized, params));
            PeakIdentification {
                peak: *peak,
                candidates,
            }
        })
        .collect()
}

pub fn best_match(identification: &PeakIdentification) -> Option<&NuclideMatch> {
    identification.candidates.first()
}
