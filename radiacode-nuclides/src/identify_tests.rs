use crate::identify::{AnalysisWindow, analyze_peaks};
use crate::match_peaks::{MatchParams, SpectrumPeak};

#[test]
fn thoriated_fixture_prefers_thorium_series() {
    let peaks = thorium_fixture_peaks();
    let params = MatchParams::default();
    let window = AnalysisWindow::from_peaks(&peaks);
    let result = analyze_peaks(&peaks, params, window);
    let top_chain = result
        .sources
        .chains
        .first()
        .expect("expected at least one chain");
    assert!(
        top_chain.name.contains("Thorium"),
        "expected thorium series first, got {top_chain:?} in {:?}",
        result.sources.chains
    );
    assert!(
        top_chain.matched_members >= 3,
        "expected >=3 thorium members, got {top_chain:?}"
    );
    assert!(
        result.sources.verdict.headline.contains("Thorium"),
        "verdict headline: {}",
        result.sources.verdict.headline
    );
}

#[test]
fn ag108m_rejected_at_727_without_supporting_lines() {
    let peaks = vec![SpectrumPeak {
        energy_kev: 722.7,
        counts: 100.0,
    }];
    let params = MatchParams::default();
    let window = AnalysisWindow::default();
    let result = analyze_peaks(&peaks, params, window);
    let best = result.identifications[0]
        .candidates
        .first()
        .map(|candidate| candidate.display_name.as_str())
        .unwrap_or("");
    assert_ne!(
        best, "Ag-108m",
        "Ag-108m should not win without 434/614 keV lines"
    );
}

fn thorium_fixture_peaks() -> Vec<SpectrumPeak> {
    vec![
        SpectrumPeak {
            energy_kev: 239.5,
            counts: 120.0,
        },
        SpectrumPeak {
            energy_kev: 338.3,
            counts: 80.0,
        },
        SpectrumPeak {
            energy_kev: 583.8,
            counts: 200.0,
        },
        SpectrumPeak {
            energy_kev: 722.7,
            counts: 60.0,
        },
        SpectrumPeak {
            energy_kev: 911.2,
            counts: 90.0,
        },
        SpectrumPeak {
            energy_kev: 969.0,
            counts: 70.0,
        },
        SpectrumPeak {
            energy_kev: 2614.5,
            counts: 150.0,
        },
    ]
}
