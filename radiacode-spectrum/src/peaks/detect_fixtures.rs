use serde::Deserialize;

use crate::peaks::DetectedPeak;
use crate::peaks::resolution::fwhm_kev;

#[derive(Debug, Deserialize)]
pub struct FixtureSpectrum {
    pub energies_kev: Vec<f64>,
    pub counts: Vec<u64>,
}

pub fn load_fixture(name: &str) -> FixtureSpectrum {
    let raw = match name {
        "thoriated_rods" => include_str!("../../data/spectra/thoriated_rods.json"),
        "background_indoor_outdoor" => {
            include_str!("../../data/spectra/background_indoor_outdoor.json")
        }
        other => panic!("unknown fixture {other}"),
    };
    serde_json::from_str(raw).expect("fixture json")
}

pub fn fixture_counts(fixture: &FixtureSpectrum) -> Vec<f64> {
    fixture.counts.iter().map(|&value| value as f64).collect()
}

pub fn has_peak_near(peaks: &[DetectedPeak], energy_kev: f64, fwhm_pct: f64) -> bool {
    let tolerance = 0.5 * fwhm_kev(energy_kev, fwhm_pct);
    peaks
        .iter()
        .any(|peak| (peak.energy_kev - energy_kev).abs() <= tolerance)
}
