use crate::app_config::AppConfig;
use crate::identify::{analysis_window_from_energies, analyze_peaks, match_params_from_config};
use crate::peaks::detect_fixtures::{fixture_counts, load_fixture};
use crate::peaks::{DetectionParams, detect_peaks};
use radiacode_nuclides::{VerdictKind, tolerance_kev};

#[test]
fn thoriated_rods_identifies_thorium_series() {
    let fixture = load_fixture("thoriated_rods");
    let params = DetectionParams::default();
    let peaks = detect_peaks(&fixture.energies_kev, &fixture_counts(&fixture), params);
    let config = AppConfig::default();
    let window = analysis_window_from_energies(&fixture.energies_kev);
    let analysis = analyze_peaks(&peaks, &config, window);
    let top_chain = analysis
        .sources
        .chains
        .first()
        .expect("expected at least one chain");
    assert!(
        top_chain.name.contains("Thorium"),
        "expected thorium series first, got {top_chain:?} in {:?}",
        analysis.sources.chains
    );
    assert!(
        top_chain.matched_members >= 3,
        "expected >=3 thorium members, got {top_chain:?}"
    );
    assert!(
        analysis.sources.verdict.headline.contains("Thorium"),
        "verdict headline: {}",
        analysis.sources.verdict.headline
    );
}

#[test]
fn background_identifies_k40_nuclide() {
    let fixture = load_fixture("background_indoor_outdoor");
    let params = DetectionParams::default();
    let peaks = detect_peaks(&fixture.energies_kev, &fixture_counts(&fixture), params);
    let config = AppConfig::default();
    let window = analysis_window_from_energies(&fixture.energies_kev);
    let analysis = analyze_peaks(&peaks, &config, window);
    assert_ne!(
        analysis.sources.verdict.kind,
        VerdictKind::Chain,
        "background should not classify as a junk chain: {}",
        analysis.sources.verdict.headline
    );
    assert!(
        analysis
            .sources
            .nuclides
            .iter()
            .any(|nuclide| nuclide.display_name.starts_with("K-40")),
        "expected K-40 in nuclide evidence {:?}",
        analysis.sources.nuclides
    );
}

#[test]
fn detector_fwhm_slider_changes_match_tolerance() {
    let narrow = AppConfig {
        detector_fwhm_pct: 4.0,
        ..Default::default()
    };
    let wide = AppConfig {
        detector_fwhm_pct: 15.0,
        ..Default::default()
    };
    let narrow_tol = tolerance_kev(727.0, match_params_from_config(&narrow));
    let wide_tol = tolerance_kev(727.0, match_params_from_config(&wide));
    assert!(wide_tol > narrow_tol);
}
