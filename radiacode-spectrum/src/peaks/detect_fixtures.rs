use serde::Deserialize;

use crate::app_config::AppConfig;
use crate::identify::{PeakAnalysis, analysis_window_from_energies, analyze_peaks};
use crate::model::SpectrumView;
use crate::peaks::resolution::fwhm_kev;
use crate::peaks::{DetectedPeak, DetectionParams, detect_peaks};

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

pub fn fixture_spectrum_view(name: &str, duration_secs: u64) -> SpectrumView {
    let fixture = load_fixture(name);
    let a1 = (fixture.energies_kev[1] - fixture.energies_kev[0]) as f32;
    SpectrumView {
        duration: std::time::Duration::from_secs(duration_secs),
        a0: fixture.energies_kev[0] as f32,
        a1,
        a2: 0.0,
        total_counts: fixture.counts.iter().sum(),
        counts: fixture.counts.iter().map(|&value| value as u32).collect(),
    }
}

pub fn fixture_analysis(name: &str) -> PeakAnalysis {
    let fixture = load_fixture(name);
    let peaks = detect_peaks(
        &fixture.energies_kev,
        &fixture_counts(&fixture),
        DetectionParams::default(),
    );
    let window = analysis_window_from_energies(&fixture.energies_kev);
    analyze_peaks(&peaks, &AppConfig::default(), window)
}
