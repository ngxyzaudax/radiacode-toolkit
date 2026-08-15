use std::collections::HashMap;

use crate::catalog::{energy_index, nuclide_by_id};
use crate::index::IndexedLine;
use crate::model::NuclideId;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MatchParams {
    pub relative_frac: f64,
    pub floor_kev: f64,
    pub min_intensity_pct: f64,
}

impl Default for MatchParams {
    fn default() -> Self {
        Self {
            relative_frac: 0.01,
            floor_kev: 3.0,
            min_intensity_pct: 1.0,
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
    (energy_kev * params.relative_frac).max(params.floor_kev)
}

pub fn match_peaks(peaks: &[SpectrumPeak], params: MatchParams) -> Vec<PeakIdentification> {
    if peaks.is_empty() {
        return Vec::new();
    }
    let per_peak = peaks
        .iter()
        .map(|peak| single_peak_candidates(*peak, params))
        .collect::<Vec<_>>();
    let nuclide_scores = aggregate_nuclide_scores(&per_peak);
    peaks
        .iter()
        .zip(per_peak)
        .map(|(peak, candidates)| {
            let ranked = rank_candidates(candidates, &nuclide_scores);
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
    let intensity_weight = (line.intensity_pct / 100.0).clamp(0.01, 1.0);
    let score = closeness + intensity_weight * 0.001;
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

fn aggregate_nuclide_scores(
    per_peak: &[Vec<NuclideMatch>],
) -> HashMap<NuclideId, (f64, u32)> {
    let mut totals = HashMap::new();
    for candidates in per_peak {
        let mut best_per_nuclide = HashMap::new();
        for candidate in candidates {
            best_per_nuclide
                .entry(candidate.nuclide_id)
                .and_modify(|existing: &mut NuclideMatch| {
                    if candidate.score > existing.score {
                        *existing = candidate.clone();
                    }
                })
                .or_insert_with(|| candidate.clone());
        }
        for candidate in best_per_nuclide.into_values() {
            let entry = totals
                .entry(candidate.nuclide_id)
                .or_insert((0.0_f64, 0_u32));
            entry.0 += candidate.score;
            entry.1 += 1;
        }
    }
    totals
}

fn rank_candidates(
    mut candidates: Vec<NuclideMatch>,
    nuclide_scores: &HashMap<NuclideId, (f64, u32)>,
) -> Vec<NuclideMatch> {
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
    for candidate in &mut ranked {
        if let Some((total, count)) = nuclide_scores.get(&candidate.nuclide_id) {
            let boost = 1.0 + 0.5 * (*count as f64 - 1.0).max(0.0);
            candidate.score = candidate.score * boost + total * 0.25;
            candidate.matched_lines = *count;
        }
    }
    ranked.sort_by(|left, right| {
        right
            .score
            .partial_cmp(&left.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    ranked
}

pub fn best_match(identification: &PeakIdentification) -> Option<&NuclideMatch> {
    identification.candidates.first()
}
