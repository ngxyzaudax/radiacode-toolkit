use radiacode_nuclides::{AttributedLine, GammaLine};

use crate::energy::energy_grid;
use crate::model::SpectrumView;

pub const NORMALIZED_PEAK: f64 = 100.0;
const MEASURED_NORMALIZE_FROM_KEV: f64 = 30.0;

pub fn scaled_gamma_lines(lines: &[AttributedLine]) -> Vec<GammaLine> {
    lines
        .iter()
        .map(|line| GammaLine {
            intensity_pct: line.scaled_intensity_pct,
            ..line.line.clone()
        })
        .collect()
}

pub fn measured_profile(spectrum: &SpectrumView, max_energy_kev: f64) -> (Vec<f64>, Vec<f64>) {
    let grid = energy_grid(spectrum);
    let (energies, counts): (Vec<f64>, Vec<f64>) = grid
        .indices
        .iter()
        .zip(grid.energies_kev.iter())
        .filter(|(_, energy)| **energy <= max_energy_kev)
        .map(|(&channel, &energy)| (energy, f64::from(spectrum.counts[channel])))
        .unzip();
    let normalized = normalize_above(&energies, &counts, MEASURED_NORMALIZE_FROM_KEV);
    (energies, normalized)
}

fn normalize_above(energies: &[f64], counts: &[f64], min_energy_kev: f64) -> Vec<f64> {
    let peak = energies
        .iter()
        .zip(counts.iter())
        .filter(|(energy, _)| **energy >= min_energy_kev)
        .map(|(_, count)| *count)
        .fold(0.0_f64, f64::max);
    if peak <= 0.0 {
        return vec![0.0; counts.len()];
    }
    counts
        .iter()
        .map(|count| (count / peak * NORMALIZED_PEAK).min(NORMALIZED_PEAK))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{measured_profile, normalize_above, scaled_gamma_lines};
    use crate::model::SpectrumView;
    use radiacode_nuclides::{AttributedLine, DecayMode, GammaLine, NuclideId, RadiationKind};

    fn spectrum() -> SpectrumView {
        SpectrumView {
            duration: std::time::Duration::from_secs(10),
            a0: 0.0,
            a1: 3.0,
            a2: 0.0,
            counts: (0..1024)
                .map(|channel| if channel == 220 { 500 } else { 5 })
                .collect(),
            total_counts: 0,
        }
    }

    #[test]
    fn scaled_lines_carry_attributed_intensity() {
        let line = AttributedLine {
            line: GammaLine {
                energy_kev: 609.3,
                intensity_pct: 45.0,
                decay: DecayMode::BetaMinus,
                kind: RadiationKind::Gamma,
            },
            source: NuclideId::new(83, 131, 0),
            source_name: "Bi-214".into(),
            scaled_intensity_pct: 22.5,
        };
        assert_eq!(scaled_gamma_lines(&[line])[0].intensity_pct, 22.5);
    }

    #[test]
    fn measured_profile_is_cut_and_normalized() {
        let (energies, values) = measured_profile(&spectrum(), 1000.0);
        assert!(energies.iter().all(|energy| *energy <= 1000.0));
        let peak = values.iter().copied().fold(0.0, f64::max);
        assert_eq!(peak, 100.0);
    }

    #[test]
    fn low_energy_noise_is_clipped_to_normalized_peak() {
        let values = normalize_above(&[10.0, 100.0], &[1000.0, 10.0], 30.0);
        assert_eq!(values, vec![100.0, 100.0]);
        assert_eq!(normalize_above(&[10.0], &[0.0], 30.0), vec![0.0]);
    }
}
