#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpectrumPeak {
    pub energy_kev: f64,
    pub counts: f64,
}

const BASELINE_WINDOW: usize = 41;
const PROMINENCE_HALF_WIDTH: usize = 24;
const MIN_HEIGHT_FRAC: f64 = 0.04;
const MIN_PROMINENCE_FRAC: f64 = 0.025;
const MIN_CHANNEL_SEPARATION: usize = 4;

pub fn detect_peaks(energies_kev: &[f64], counts: &[f64]) -> Vec<SpectrumPeak> {
    if energies_kev.len() < 3 || energies_kev.len() != counts.len() {
        return Vec::new();
    }
    let residual = continuum_residual(counts);
    let max_residual = residual.iter().copied().fold(0.0_f64, f64::max);
    if max_residual <= 0.0 {
        return Vec::new();
    }
    let min_height = max_residual * MIN_HEIGHT_FRAC;
    let min_prominence = max_residual * MIN_PROMINENCE_FRAC;
    let mut candidates = local_maxima(&residual)
        .into_iter()
        .filter(|&index| residual[index] >= min_height)
        .filter(|&index| windowed_prominence(&residual, index) >= min_prominence)
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| {
        residual[*right]
            .partial_cmp(&residual[*left])
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let picked = pick_separated(candidates, &residual);
    let mut peaks = picked
        .into_iter()
        .map(|index| SpectrumPeak {
            energy_kev: energies_kev[index],
            counts: counts[index],
        })
        .collect::<Vec<_>>();
    peaks.sort_by(|left, right| {
        left.energy_kev
            .partial_cmp(&right.energy_kev)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    peaks
}

pub fn peaks_in_energy_range(peaks: &[SpectrumPeak], min_kev: f64, max_kev: f64) -> Vec<SpectrumPeak> {
    peaks
        .iter()
        .copied()
        .filter(|peak| peak.energy_kev >= min_kev && peak.energy_kev <= max_kev)
        .collect()
}

fn continuum_residual(counts: &[f64]) -> Vec<f64> {
    let baseline = crate::smooth::moving_average_f64(counts, BASELINE_WINDOW);
    counts
        .iter()
        .zip(baseline.iter())
        .map(|(value, base)| (value - base).max(0.0))
        .collect()
}

fn local_maxima(counts: &[f64]) -> Vec<usize> {
    (1..counts.len() - 1)
        .filter(|&index| counts[index] >= counts[index - 1] && counts[index] > counts[index + 1])
        .collect()
}

fn windowed_prominence(counts: &[f64], index: usize) -> f64 {
    let peak = counts[index];
    let left = index.saturating_sub(PROMINENCE_HALF_WIDTH);
    let right = (index + PROMINENCE_HALF_WIDTH + 1).min(counts.len());
    let left_base = base_in_range(counts, left, index);
    let right_base = base_in_range(counts, index + 1, right);
    peak - left_base.max(right_base)
}

fn base_in_range(counts: &[f64], start: usize, end: usize) -> f64 {
    counts[start..end].iter().copied().fold(f64::INFINITY, f64::min)
}

fn pick_separated(candidates: Vec<usize>, counts: &[f64]) -> Vec<usize> {
    let mut picked = Vec::new();
    'outer: for index in candidates {
        for chosen in &picked {
            if index.abs_diff(*chosen) < MIN_CHANNEL_SEPARATION {
                continue 'outer;
            }
        }
        picked.push(index);
    }
    picked.sort_by(|left, right| {
        counts[*left]
            .partial_cmp(&counts[*right])
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    picked
}

#[cfg(test)]
mod tests {
    use super::{detect_peaks, peaks_in_energy_range};

    #[test]
    fn finds_two_separated_peaks() {
        let mut counts = vec![1.0; 20];
        counts[5] = 40.0;
        counts[6] = 55.0;
        counts[7] = 42.0;
        counts[14] = 30.0;
        counts[15] = 35.0;
        counts[16] = 28.0;
        let energies: Vec<f64> = (0..20).map(|index| index as f64 * 10.0).collect();
        let peaks = detect_peaks(&energies, &counts);
        assert_eq!(peaks.len(), 2);
        assert!((peaks[0].energy_kev - 60.0).abs() < 0.1);
        assert!((peaks[1].energy_kev - 150.0).abs() < 0.1);
    }

    #[test]
    fn finds_peaks_on_sloping_compton_continuum() {
        let length = 1024;
        let energies: Vec<f64> = (0..length).map(|index| 5.45 + index as f64 * 2.375).collect();
        let mut counts: Vec<f64> = (0..length)
            .map(|index| 20_000.0 * (-(index as f64) / 120.0).exp() + 80.0)
            .collect();
        let cs_index = energies
            .iter()
            .position(|energy| (*energy - 661.7).abs() < 2.0)
            .expect("cs channel");
        let k_index = energies
            .iter()
            .position(|energy| (*energy - 1460.0).abs() < 3.0)
            .expect("k channel");
        counts[cs_index] += 350.0;
        if cs_index + 1 < length {
            counts[cs_index + 1] += 280.0;
        }
        if cs_index > 0 {
            counts[cs_index - 1] += 220.0;
        }
        counts[k_index] += 180.0;
        if k_index + 1 < length {
            counts[k_index + 1] += 150.0;
        }
        if k_index > 0 {
            counts[k_index - 1] += 120.0;
        }
        let peaks = detect_peaks(&energies, &counts);
        let has_cs = peaks.iter().any(|peak| (peak.energy_kev - 661.7).abs() < 15.0);
        let has_k = peaks.iter().any(|peak| (peak.energy_kev - 1460.0).abs() < 20.0);
        assert!(has_cs, "expected Cs-137 region peak, got {peaks:?}");
        assert!(has_k, "expected K-40 region peak, got {peaks:?}");
    }

    #[test]
    fn ignores_flat_noise() {
        let counts = vec![5.0; 32];
        let energies: Vec<f64> = (0..32).map(|index| index as f64).collect();
        assert!(detect_peaks(&energies, &counts).is_empty());
    }

    #[test]
    fn filters_by_energy_window() {
        let peaks = vec![
            super::SpectrumPeak {
                energy_kev: 100.0,
                counts: 10.0,
            },
            super::SpectrumPeak {
                energy_kev: 500.0,
                counts: 20.0,
            },
        ];
        let visible = peaks_in_energy_range(&peaks, 50.0, 300.0);
        assert_eq!(visible.len(), 1);
        assert!((visible[0].energy_kev - 100.0).abs() < 0.1);
    }
}
