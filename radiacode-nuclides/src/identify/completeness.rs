use crate::catalog::nuclide_by_id;
use crate::identify::window::AnalysisWindow;
use crate::match_peaks::{MatchParams, PeakIdentification, tolerance_kev};
use crate::model::{NuclideId, RadiationKind};

const ANNIHILATION_KEV: f64 = 511.0;
const STRONG_LINE_MIN_INTENSITY_PCT: f64 = 15.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CompletenessScore {
    pub explained: u32,
    pub missing: u32,
    pub penalty: f64,
}

pub fn completeness_for_nuclide(
    id: NuclideId,
    identifications: &[PeakIdentification],
    window: AnalysisWindow,
    params: MatchParams,
) -> CompletenessScore {
    let Some(nuclide) = nuclide_by_id(id) else {
        return CompletenessScore {
            explained: 0,
            missing: 0,
            penalty: 0.0,
        };
    };
    let mut explained = 0u32;
    let mut missing = 0u32;
    for gamma in &nuclide.gammas {
        if gamma.kind != RadiationKind::Gamma {
            continue;
        }
        if !counts_for_completeness(gamma.energy_kev, gamma.intensity_pct, window, params) {
            continue;
        }
        if peak_explains_line(gamma.energy_kev, identifications, params) {
            explained += 1;
        } else {
            missing += 1;
        }
    }
    let penalty = missing_line_penalty(id, identifications, window, params);
    CompletenessScore {
        explained,
        missing,
        penalty,
    }
}

fn counts_for_completeness(
    energy_kev: f64,
    intensity_pct: f64,
    window: AnalysisWindow,
    params: MatchParams,
) -> bool {
    if intensity_pct < STRONG_LINE_MIN_INTENSITY_PCT {
        return false;
    }
    if energy_kev < params.low_energy_cutoff_kev {
        return false;
    }
    if (energy_kev - ANNIHILATION_KEV).abs() < 5.0 {
        return false;
    }
    window.contains(energy_kev)
}

fn missing_line_penalty(
    id: NuclideId,
    identifications: &[PeakIdentification],
    window: AnalysisWindow,
    params: MatchParams,
) -> f64 {
    let Some(nuclide) = nuclide_by_id(id) else {
        return 0.0;
    };
    nuclide
        .gammas
        .iter()
        .filter(|gamma| gamma.kind == RadiationKind::Gamma)
        .filter(|gamma| {
            counts_for_completeness(gamma.energy_kev, gamma.intensity_pct, window, params)
        })
        .filter(|gamma| !peak_explains_line(gamma.energy_kev, identifications, params))
        .map(|gamma| (gamma.intensity_pct / 100.0) * params.absence_penalty)
        .sum()
}

fn peak_explains_line(
    line_energy_kev: f64,
    identifications: &[PeakIdentification],
    params: MatchParams,
) -> bool {
    let tolerance = tolerance_kev(line_energy_kev, params);
    identifications
        .iter()
        .any(|identification| (identification.peak.energy_kev - line_energy_kev).abs() <= tolerance)
}

const HYPOTHESIS_MAX_RANK: usize = 3;

pub fn nuclide_hypothesis_score(
    id: NuclideId,
    identifications: &[PeakIdentification],
    window: AnalysisWindow,
    params: MatchParams,
) -> f64 {
    let peak_score: f64 = identifications
        .iter()
        .filter_map(|identification| {
            identification
                .candidates
                .iter()
                .take(HYPOTHESIS_MAX_RANK)
                .enumerate()
                .find_map(|(rank, candidate)| {
                    if candidate.nuclide_id == id {
                        Some(candidate.score / (rank as f64 + 1.0))
                    } else {
                        None
                    }
                })
        })
        .sum();
    if peak_score <= 0.0 {
        return 0.0;
    }
    let completeness = completeness_for_nuclide(id, identifications, window, params);
    (peak_score - completeness.penalty).max(0.0)
}
