use std::collections::HashMap;

use crate::catalog::{energy_index, nuclide_by_id};
use crate::index::IndexedLine;
use crate::model::NuclideId;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MatchParams {
    pub relative_frac: f64,
    pub floor_kev: f64,
    pub min_intensity_pct: f64,
    pub detector_fwhm_pct: f64,
    pub tolerance_fwhm_frac: f64,
}

impl Default for MatchParams {
    fn default() -> Self {
        Self {
            relative_frac: 0.02,
            floor_kev: 3.0,
            min_intensity_pct: 1.7,
            detector_fwhm_pct: 7.0,
            tolerance_fwhm_frac: 0.5,
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
    let normalized = peaks
        .iter()
        .map(|peak| SpectrumPeak {
            energy_kev: peak.energy_kev,
            counts: peak.counts / max_counts,
        })
        .collect::<Vec<_>>();
    let per_peak = normalized
        .iter()
        .map(|peak| single_peak_candidates(*peak, params))
        .collect::<Vec<_>>();
    normalized
        .iter()
        .zip(peaks.iter())
        .zip(per_peak)
        .map(|((_, peak), candidates)| {
            let ranked = rank_candidates(candidates);
            PeakIdentification {
                peak: *peak,
                candidates: ranked,
            }
        })
        .collect()
}

fn single_peak_candidates(peak: SpectrumPeak, params: MatchParams) -> Vec<NuclideMatch> {
    let tolerance = tolerance_kev(peak.energy_kev, params);
    let min_energy = peak.energy_kev - tolerance;
    let max_energy = peak.energy_kev + tolerance;
    energy_index()
        .range(min_energy, max_energy)
        .iter()
        .filter(|line| line.intensity_pct >= params.min_intensity_pct)
        .filter_map(|line| line_match(peak, line, tolerance))
        .collect()
}

fn line_match(peak: SpectrumPeak, line: &IndexedLine, tolerance: f64) -> Option<NuclideMatch> {
    let nuclide = nuclide_by_id(line.nuclide_id)?;
    let delta = (peak.energy_kev - line.energy_kev).abs();
    if delta > tolerance {
        return None;
    }
    let closeness = 1.0 - delta / tolerance;
    if closeness < 0.25 {
        return None;
    }
    let intensity_weight = (line.intensity_pct / 100.0).clamp(0.01, 1.0);
    let count_weight = (peak.counts.max(1e-6).sqrt() + 0.1).min(2.0);
    let score = closeness * closeness * (0.15 + 0.85 * intensity_weight) * count_weight;
    Some(NuclideMatch {
        nuclide_id: line.nuclide_id,
        display_name: nuclide.display_name.clone(),
        line_energy_kev: line.energy_kev,
        intensity_pct: line.intensity_pct,
        delta_kev: delta,
        score,
        matched_lines: 1,
    })
}

fn rank_candidates(mut candidates: Vec<NuclideMatch>) -> Vec<NuclideMatch> {
    let mut best_by_nuclide = HashMap::new();
    for candidate in candidates.drain(..) {
        best_by_nuclide
            .entry(candidate.nuclide_id)
            .and_modify(|existing: &mut NuclideMatch| {
                if candidate.score > existing.score {
                    *existing = candidate.clone();
                }
            })
            .or_insert(candidate);
    }
    let mut ranked = best_by_nuclide.into_values().collect::<Vec<_>>();
    ranked.sort_by(|left, right| {
        right
            .score
            .partial_cmp(&left.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                left.delta_kev
                    .partial_cmp(&right.delta_kev)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| left.display_name.cmp(&right.display_name))
    });
    ranked
}

pub fn best_match(identification: &PeakIdentification) -> Option<&NuclideMatch> {
    identification.candidates.first()
}
