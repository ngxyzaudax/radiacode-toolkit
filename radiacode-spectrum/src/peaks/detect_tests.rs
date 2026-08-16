use serde::Deserialize;

use crate::peaks::resolution::fwhm_kev;
use crate::peaks::{DetectionParams, detect_peaks};
use crate::identify::{analyze_peaks, detection_params_from_config, match_params_from_config};
use crate::app_config::AppConfig;
use radiacode_nuclides::tolerance_kev;

#[derive(Debug, Deserialize)]
struct FixtureSpectrum {
    energies_kev: Vec<f64>,
    counts: Vec<u64>,
}

fn load_fixture(name: &str) -> FixtureSpectrum {
    let raw = match name {
        "thoriated_rods" => include_str!("../../data/spectra/thoriated_rods.json"),
        "background_indoor_outdoor" => {
            include_str!("../../data/spectra/background_indoor_outdoor.json")
        }
        other => panic!("unknown fixture {other}"),
    };
    serde_json::from_str(raw).expect("fixture json")
}

fn fixture_counts(fixture: &FixtureSpectrum) -> Vec<f64> {
    fixture.counts.iter().map(|&value| value as f64).collect()
}

fn has_peak_near(peaks: &[crate::peaks::DetectedPeak], energy_kev: f64, fwhm_pct: f64) -> bool {
    let tolerance = 0.5 * fwhm_kev(energy_kev, fwhm_pct);
    peaks
        .iter()
        .any(|peak| (peak.energy_kev - energy_kev).abs() <= tolerance)
}

#[test]
fn thoriated_rods_finds_thorium_lines() {
    let fixture = load_fixture("thoriated_rods");
    let params = DetectionParams::default();
    let peaks = detect_peaks(
        &fixture.energies_kev,
        &fixture_counts(&fixture),
        params,
    );
    for energy in [238.6, 338.3, 583.2, 727.3, 911.2, 2614.5] {
        assert!(
            has_peak_near(&peaks, energy, params.detector_fwhm_pct),
            "missing peak near {energy} keV in {peaks:?}"
        );
    }
}

#[test]
fn background_finds_potassium() {
    let fixture = load_fixture("background_indoor_outdoor");
    let params = DetectionParams::default();
    let peaks = detect_peaks(
        &fixture.energies_kev,
        &fixture_counts(&fixture),
        params,
    );
    assert!(
        has_peak_near(&peaks, 1460.8, params.detector_fwhm_pct),
        "missing K-40 peak in {peaks:?}"
    );
}

#[test]
fn thoriated_rods_identifies_thorium_series() {
    let fixture = load_fixture("thoriated_rods");
    let params = DetectionParams::default();
    let peaks = detect_peaks(
        &fixture.energies_kev,
        &fixture_counts(&fixture),
        params,
    );
    let config = AppConfig::default();
    let analysis = analyze_peaks(&peaks, &config);
    assert!(
        analysis
            .sources
            .chains
            .iter()
            .any(|chain| chain.name.contains("Thorium")),
        "expected thorium series in {:?}",
        analysis.sources.chains
    );
}

#[test]
fn flat_noise_has_no_peaks() {
    let energies: Vec<f64> = (0..256).map(|index| index as f64 * 10.0).collect();
    let counts = vec![5.0; 256];
    let peaks = detect_peaks(&energies, &counts, DetectionParams::default());
    assert!(peaks.is_empty());
}

#[test]
fn detection_params_follow_app_config() {
    let mut config = AppConfig::default();
    config.peak_sensitivity_sigma = 5.7;
    config.detector_fwhm_pct = 8.5;
    config.match_tolerance_frac = 0.02;
    config.match_min_intensity_pct = 1.7;
    let params = detection_params_from_config(&config);
    assert_eq!(params.sigma_min, 5.7);
    assert_eq!(params.detector_fwhm_pct, 8.5);
    let match_params = match_params_from_config(&config);
    assert_eq!(match_params.detector_fwhm_pct, 8.5);
    assert_eq!(match_params.relative_frac, 0.02);
    assert_eq!(match_params.min_intensity_pct, 1.7);
}

#[test]
fn sensitivity_slider_changes_peak_count() {
    let fixture = load_fixture("thoriated_rods");
    let counts = fixture_counts(&fixture);
    let mut strict = AppConfig::default();
    strict.peak_sensitivity_sigma = 15.0;
    let mut sensitive = AppConfig::default();
    sensitive.peak_sensitivity_sigma = 2.0;
    let strict_peaks = detect_peaks(
        &fixture.energies_kev,
        &counts,
        detection_params_from_config(&strict),
    );
    let sensitive_peaks = detect_peaks(
        &fixture.energies_kev,
        &counts,
        detection_params_from_config(&sensitive),
    );
    assert!(
        sensitive_peaks.len() > strict_peaks.len(),
        "strict {} peaks, sensitive {} peaks",
        strict_peaks.len(),
        sensitive_peaks.len()
    );
}

#[test]
fn detector_fwhm_slider_changes_match_tolerance() {
    let mut narrow = AppConfig::default();
    narrow.detector_fwhm_pct = 4.0;
    let mut wide = AppConfig::default();
    wide.detector_fwhm_pct = 15.0;
    let narrow_tol = tolerance_kev(727.0, match_params_from_config(&narrow));
    let wide_tol = tolerance_kev(727.0, match_params_from_config(&wide));
    assert!(wide_tol > narrow_tol);
}

#[test]
fn tuned_settings_find_thorium_lines() {
    let fixture = load_fixture("thoriated_rods");
    let mut config = AppConfig::default();
    config.peak_sensitivity_sigma = 5.7;
    config.detector_fwhm_pct = 8.5;
    config.match_tolerance_frac = 0.02;
    config.match_tolerance_floor_kev = 3.0;
    config.match_min_intensity_pct = 1.7;
    let params = detection_params_from_config(&config);
    let peaks = detect_peaks(
        &fixture.energies_kev,
        &fixture_counts(&fixture),
        params,
    );
    for energy in [238.6, 338.3, 583.2, 727.3, 911.2, 2614.5] {
        assert!(
            has_peak_near(&peaks, energy, params.detector_fwhm_pct),
            "missing peak near {energy} keV with tuned settings in {peaks:?}"
        );
    }
    let analysis = analyze_peaks(&peaks, &config);
    assert!(
        analysis
            .sources
            .chains
            .iter()
            .any(|chain| chain.name.contains("Thorium")),
        "expected thorium series with tuned settings in {:?}",
        analysis.sources.chains
    );
}
