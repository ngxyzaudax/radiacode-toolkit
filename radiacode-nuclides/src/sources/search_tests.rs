use super::{
    EnergyQuery, closeness, confirm_other_lines, match_nuclides_at_energy, match_sources_at_energy,
    source_by_key,
};

fn query(energy_kev: f64, peaks_kev: &[f64]) -> EnergyQuery<'_> {
    EnergyQuery {
        energy_kev,
        fwhm_pct: 7.0,
        peaks_kev,
    }
}

fn top_key(energy_kev: f64, peaks_kev: &[f64]) -> &'static str {
    match_sources_at_energy(query(energy_kev, peaks_kev))[0]
        .source
        .key
}

#[test]
fn energy_search_ranks_common_source_first_without_spectrum() {
    assert_eq!(top_key(662.0, &[]), "cs137");
    assert_eq!(top_key(60.0, &[]), "am241");
    assert_eq!(top_key(1461.0, &[]), "k40");
}

#[test]
fn energy_search_returns_nothing_outside_resolution() {
    assert!(match_sources_at_energy(query(5000.0, &[])).is_empty());
}

#[test]
fn missing_companion_lines_demote_candidates() {
    let peaks = [662.0];
    let results = match_sources_at_energy(query(662.0, &peaks));
    let silver = results
        .iter()
        .find(|entry| entry.source.key == "ag110m")
        .expect("Ag-110m candidate");
    let confirmation = silver.confirmation.expect("confirmation");
    assert_eq!(confirmation.found, 0);
    assert!(confirmation.expected >= 2);
    assert_eq!(results[0].source.key, "cs137");
}

#[test]
fn present_companion_lines_promote_candidates() {
    let peaks = [1173.2, 1332.5];
    let results = match_sources_at_energy(query(1173.0, &peaks));
    let cobalt = &results[0];
    assert_eq!(cobalt.source.key, "co60");
    let confirmation = cobalt.confirmation.expect("confirmation");
    assert_eq!(confirmation.found, confirmation.expected);
}

#[test]
fn confirmation_counts_major_lines_except_matched_one() {
    let thorium = source_by_key("th232-series").expect("thorium");
    let confirmation = confirm_other_lines(thorium, 2614.5, &[238.6, 583.2, 911.2], 7.0);
    assert!(confirmation.found >= 2);
    assert!(confirmation.expected >= confirmation.found);
    assert!(confirmation.factor() > 0.0);
}

#[test]
fn closeness_is_half_at_half_fwhm() {
    assert!((closeness(5.0, 10.0) - 0.5).abs() < 1e-9);
    assert_eq!(closeness(1.0, 0.0), 0.0);
}

#[test]
fn nuclide_search_finds_long_lived_gamma_emitters_only() {
    let matches = match_nuclides_at_energy(1332.0, 7.0, 86_400.0);
    assert!(
        matches
            .iter()
            .any(|entry| entry.nuclide.display_name == "Co-60")
    );
    assert!(matches.iter().all(|entry| entry.line.intensity_pct >= 1.0));
}
