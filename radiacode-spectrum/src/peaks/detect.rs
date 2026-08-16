use crate::peaks::continuum::{snip_baseline, snip_iterations};
use crate::peaks::matched_filter::gaussian_smooth;
use crate::peaks::model::{DetectedPeak, DetectionParams};
use crate::peaks::resolution::{fwhm_channels, fwhm_kev};

pub fn detect_peaks(energies_kev: &[f64], counts: &[f64], params: DetectionParams) -> Vec<DetectedPeak> {
    if energies_kev.len() < 5 || energies_kev.len() != counts.len() {
        return Vec::new();
    }
    let step = energy_step(energies_kev);
    let mid = energies_kev[energies_kev.len() / 2];
    let iterations = snip_iterations(step, mid, params.detector_fwhm_pct);
    let baseline = snip_baseline(counts, iterations);
    let net: Vec<f64> = counts
        .iter()
        .zip(baseline.iter())
        .map(|(value, base)| (value - base).max(0.0))
        .collect();
    let filtered = gaussian_smooth(&net, energies_kev, params.detector_fwhm_pct);
    let mut candidates = Vec::new();
    for index in 2..net.len().saturating_sub(2) {
        let half_width = fwhm_channels(energies_kev[index], step, params.detector_fwhm_pct) / 2;
        let half_width = half_width.max(1);
        let start = index.saturating_sub(half_width);
        let end = (index + half_width + 1).min(net.len());
        if filtered[index] < filtered[start..end].iter().copied().fold(0.0_f64, f64::max) {
            continue;
        }
        if net[index] <= 0.0 {
            continue;
        }
        let gross: f64 = counts[start..end].iter().sum();
        let background: f64 = baseline[start..end].iter().sum();
        let area = gross - background;
        if area <= 0.0 || background <= 0.0 {
            continue;
        }
        if area / background < params.min_net_fraction {
            continue;
        }
        let significance = area / (gross + background).sqrt();
        if significance < params.sigma_min {
            continue;
        }
        let energy = centroid_energy(&energies_kev[start..end], &net[start..end]);
        candidates.push((
            index,
            DetectedPeak {
                energy_kev: energy,
                net_area: area,
                significance,
                fwhm_kev: fwhm_kev(energy, params.detector_fwhm_pct),
            },
        ));
    }
    merge_candidates(candidates, step, params.detector_fwhm_pct)
}

fn merge_candidates(
    mut candidates: Vec<(usize, DetectedPeak)>,
    step_kev: f64,
    fwhm_pct: f64,
) -> Vec<DetectedPeak> {
    candidates.sort_by(|left, right| {
        right
            .1
            .significance
            .partial_cmp(&left.1.significance)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut kept = Vec::new();
    'outer: for (index, peak) in candidates {
        let separation = fwhm_channels(peak.energy_kev, step_kev, fwhm_pct);
        for (kept_index, _) in &kept {
            if index.abs_diff(*kept_index) < separation {
                continue 'outer;
            }
        }
        kept.push((index, peak));
    }
    let mut peaks: Vec<DetectedPeak> = kept.into_iter().map(|(_, peak)| peak).collect();
    peaks.sort_by(|left, right| {
        left.energy_kev
            .partial_cmp(&right.energy_kev)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    peaks
}

fn centroid_energy(energies: &[f64], weights: &[f64]) -> f64 {
    let total: f64 = weights.iter().sum();
    if total <= 0.0 {
        return energies[energies.len() / 2];
    }
    energies
        .iter()
        .zip(weights.iter())
        .map(|(energy, weight)| energy * weight)
        .sum::<f64>()
        / total
}

fn energy_step(energies_kev: &[f64]) -> f64 {
    if energies_kev.len() < 2 {
        return 1.0;
    }
    (energies_kev.last().copied().unwrap_or(0.0) - energies_kev.first().copied().unwrap_or(0.0))
        / (energies_kev.len() - 1) as f64
}
