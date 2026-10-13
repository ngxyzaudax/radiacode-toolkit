use crate::app_config::AppConfig;
use crate::identify::{analysis_window_from_energies, analyze_peaks, match_params_from_config};
use crate::peaks::detect_fixtures::{fixture_counts, has_peak_near, load_fixture};
use crate::peaks::{DetectionParams, detect_peaks};

#[test]
fn thoriated_rods_finds_thorium_lines() {
    let fixture = load_fixture("thoriated_rods");
    let params = DetectionParams::default();
    let peaks = detect_peaks(&fixture.energies_kev, &fixture_counts(&fixture), params);
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
    let peaks = detect_peaks(&fixture.energies_kev, &fixture_counts(&fixture), params);
    assert!(
        has_peak_near(&peaks, 1460.8, params.detector_fwhm_pct),
        "missing K-40 peak in {peaks:?}"
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
    let config = AppConfig {
        peak_sensitivity_sigma: 5.7,
        detector_fwhm_pct: 8.5,
        match_tolerance_frac: 0.02,
        match_min_intensity_pct: 1.7,
        ..Default::default()
    };
    let params = DetectionParams::from_app_config(&config);
    assert_eq!(params.sigma_min, 5.7);
    assert_eq!(params.detector_fwhm_pct, 8.5);
    let match_params = match_params_from_config(&config);
    assert_eq!(match_params.detector_fwhm_pct, 8.5);
    assert_eq!(match_params.relative_frac, 0.02);
    assert_eq!(match_params.min_intensity_pct, 1.7);
    assert_eq!(match_params.min_supported_half_life_secs, 600.0);
}

#[test]
fn sensitivity_slider_changes_peak_count() {
    let fixture = load_fixture("thoriated_rods");
    let counts = fixture_counts(&fixture);
    let strict = AppConfig {
        peak_sensitivity_sigma: 15.0,
        ..Default::default()
    };
    let sensitive = AppConfig {
        peak_sensitivity_sigma: 2.0,
        ..Default::default()
    };
    let strict_peaks = detect_peaks(
        &fixture.energies_kev,
        &counts,
        DetectionParams::from_app_config(&strict),
    );
    let sensitive_peaks = detect_peaks(
        &fixture.energies_kev,
        &counts,
        DetectionParams::from_app_config(&sensitive),
    );
    assert!(
        sensitive_peaks.len() > strict_peaks.len(),
        "strict {} peaks, sensitive {} peaks",
        strict_peaks.len(),
        sensitive_peaks.len()
    );
}

#[test]
fn tuned_settings_find_thorium_lines() {
    let fixture = load_fixture("thoriated_rods");
    let config = AppConfig {
        peak_sensitivity_sigma: 5.7,
        detector_fwhm_pct: 8.5,
        match_tolerance_frac: 0.02,
        match_tolerance_floor_kev: 3.0,
        match_min_intensity_pct: 1.7,
        ..Default::default()
    };
    let params = DetectionParams::from_app_config(&config);
    let peaks = detect_peaks(&fixture.energies_kev, &fixture_counts(&fixture), params);
    for energy in [238.6, 338.3, 583.2, 727.3, 911.2, 2614.5] {
        assert!(
            has_peak_near(&peaks, energy, params.detector_fwhm_pct),
            "missing peak near {energy} keV with tuned settings in {peaks:?}"
        );
    }
    let window = analysis_window_from_energies(&fixture.energies_kev);
    let analysis = analyze_peaks(&peaks, &config, window);
    let top_chain = analysis
        .sources
        .chains
        .first()
        .expect("expected at least one chain");
    assert!(
        top_chain.name.contains("Thorium"),
        "expected thorium series first with tuned settings, got {top_chain:?} in {:?}",
        analysis.sources.chains
    );
    assert!(top_chain.matched_members >= 3);
    assert!(analysis.sources.verdict.headline.contains("Thorium"));
}
