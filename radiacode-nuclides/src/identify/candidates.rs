use crate::catalog::nuclide_by_id;
use crate::match_peaks::{MatchParams, NuclideMatch, SpectrumPeak, tolerance_kev};
use crate::plausibility::is_plausible;
use crate::index::IndexedLine;
use crate::catalog::energy_index;

pub fn peak_candidates(peak: SpectrumPeak, params: MatchParams) -> Vec<NuclideMatch> {
    let tolerance = tolerance_kev(peak.energy_kev, params);
    let min_energy = peak.energy_kev - tolerance;
    let max_energy = peak.energy_kev + tolerance;
    energy_index()
        .range(min_energy, max_energy)
        .iter()
        .filter(|line| line.intensity_pct >= params.min_intensity_pct)
        .filter(|line| is_plausible(line.nuclide_id, params.min_supported_half_life_secs))
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

pub fn rank_candidates(mut candidates: Vec<NuclideMatch>) -> Vec<NuclideMatch> {
    use std::collections::HashMap;
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
