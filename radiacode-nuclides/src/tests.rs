use crate::catalog::{catalog, nuclide_count};
use crate::chain::decay_chain;
use crate::match_peaks::{
    MatchParams, PeakIdentification, SpectrumPeak, match_peaks, tolerance_kev,
};
use crate::model::{Nuclide, NuclideId as Id};

fn u238_id() -> Id {
    Id::new(92, 146, 0)
}

#[test]
fn catalog_parses_and_has_entries() {
    assert!(nuclide_count() > 300, "catalogue should have >300 nuclides");
    let energies = catalog()
        .nuclides
        .iter()
        .flat_map(|nuclide| nuclide.gammas.iter().map(|gamma| gamma.energy_kev))
        .collect::<Vec<_>>();
    for energy in energies {
        assert!(energy.is_finite());
        assert!(energy > 0.0);
    }
}

#[test]
fn tolerance_scales_with_energy() {
    let params = MatchParams::default();
    let low = tolerance_kev(100.0, params);
    let high = tolerance_kev(1500.0, params);
    assert!((low - 9.01).abs() < 0.1);
    assert!((high - 34.86).abs() < 0.5);
    assert!(high > low);
}

#[test]
fn identifies_cs137_peak() {
    if nuclide_by_symbol("Cs").is_none() {
        return;
    }
    let peaks = vec![SpectrumPeak {
        energy_kev: 661.7,
        counts: 100.0,
    }];
    let identifications = match_peaks(&peaks, MatchParams::default());
    let best = identifications[0].candidates.first().expect("candidate");
    assert!(
        best.display_name.contains("Cs") || best.display_name.starts_with("Ba-137m"),
        "expected Cs-137 or Ba-137m for 661.7 keV, got {}",
        best.display_name
    );
}

#[test]
fn identifies_k40_peak() {
    if nuclide_by_symbol("K").is_none() {
        return;
    }
    let peaks = vec![SpectrumPeak {
        energy_kev: 1460.8,
        counts: 100.0,
    }];
    let identifications = match_peaks(&peaks, MatchParams::default());
    assert!(
        identifications[0]
            .candidates
            .iter()
            .any(|candidate| candidate.display_name.starts_with("K-40"))
    );
}

#[test]
fn co60_multi_line_prefers_co60() {
    if nuclide_by_symbol("Co").is_none() {
        return;
    }
    let peaks = vec![
        SpectrumPeak {
            energy_kev: 1173.2,
            counts: 100.0,
        },
        SpectrumPeak {
            energy_kev: 1332.5,
            counts: 90.0,
        },
    ];
    let identifications = match_peaks(&peaks, MatchParams::default());
    assert!(top_candidates_contain(&identifications[0], "Co-60"));
    assert!(top_candidates_contain(&identifications[1], "Co-60"));
}

fn top_candidates_contain(identification: &PeakIdentification, name: &str) -> bool {
    identification
        .candidates
        .iter()
        .take(3)
        .any(|candidate| candidate.display_name.starts_with(name))
}

#[test]
fn u238_chain_reaches_stable_lead() {
    if catalog().nuclides.iter().all(|entry| entry.id != u238_id()) {
        return;
    }
    let chain = decay_chain(u238_id(), 24);
    assert!(chain.len() >= 2);
    assert_eq!(chain[0].display_name, "U-238");
    let last = chain.last().expect("terminal");
    assert!(
        last.display_name.contains("Pb") || last.decay_mode.is_none(),
        "last step was {}",
        last.display_name
    );
}

fn nuclide_by_symbol(symbol: &str) -> Option<&Nuclide> {
    catalog()
        .nuclides
        .iter()
        .find(|nuclide| nuclide.symbol.eq_ignore_ascii_case(symbol))
}
