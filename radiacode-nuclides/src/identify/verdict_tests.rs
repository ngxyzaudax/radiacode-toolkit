use super::verdict::{Confidence, VerdictKind, build_verdict};
use super::verdict_peaks::confidence_from_fraction;
use crate::match_peaks::{NuclideMatch, PeakIdentification, SpectrumPeak};
use crate::model::NuclideId;
use crate::source_summary::{ChainEvidence, NuclideEvidence};

fn identification(energy_kev: f64, counts: f64, id: NuclideId, name: &str) -> PeakIdentification {
    PeakIdentification {
        peak: SpectrumPeak { energy_kev, counts },
        candidates: vec![NuclideMatch {
            nuclide_id: id,
            display_name: name.to_string(),
            line_energy_kev: energy_kev,
            intensity_pct: 50.0,
            delta_kev: 0.0,
            score: 10.0,
            matched_lines: 1,
        }],
    }
}

fn evidence(id: NuclideId, name: &str, score: f64) -> NuclideEvidence {
    NuclideEvidence {
        id,
        display_name: name.to_string(),
        matched_lines: 1,
        score,
    }
}

fn thorium_chain(members: Vec<NuclideEvidence>) -> ChainEvidence {
    ChainEvidence {
        head: NuclideId::new(90, 142, 0),
        name: "Thorium series".into(),
        family: "4n".into(),
        matched_members: members.len() as u32,
        score: members.iter().map(|member| member.score).sum(),
        members,
    }
}

#[test]
fn confidence_thresholds() {
    assert_eq!(confidence_from_fraction(4, 5), Confidence::High);
    assert_eq!(confidence_from_fraction(2, 5), Confidence::Medium);
    assert_eq!(confidence_from_fraction(1, 5), Confidence::Low);
    assert_eq!(confidence_from_fraction(0, 0), Confidence::Low);
}

#[test]
fn empty_identifications_yield_default_verdict() {
    let verdict = build_verdict(&[], &[], &[]);
    assert_eq!(verdict.kind, VerdictKind::Inconclusive);
    assert_eq!(verdict.total_peaks, 0);
}

#[test]
fn natural_chain_with_two_explained_peaks_wins() {
    let pb212 = NuclideId::new(82, 130, 0);
    let tl208 = NuclideId::new(81, 127, 0);
    let identifications = vec![
        identification(238.6, 100.0, pb212, "Pb-212"),
        identification(2614.5, 40.0, tl208, "Tl-208"),
        identification(661.7, 10.0, NuclideId::new(55, 82, 0), "Cs-137"),
    ];
    let members = vec![
        evidence(pb212, "Pb-212", 5.0),
        evidence(tl208, "Tl-208", 4.0),
    ];
    let chains = vec![thorium_chain(members.clone())];
    let verdict = build_verdict(&identifications, &chains, &members);
    assert_eq!(verdict.kind, VerdictKind::Chain);
    assert_eq!(verdict.explained_peaks, 2);
    assert_eq!(verdict.chain_head, Some(NuclideId::new(90, 142, 0)));
    assert!(verdict.headline.starts_with("Thorium series - 2 of 3"));
}

#[test]
fn single_nuclide_verdict_follows_strongest_peak() {
    let cs137 = NuclideId::new(55, 82, 0);
    let identifications = vec![identification(661.7, 500.0, cs137, "Cs-137")];
    let nuclides = vec![evidence(cs137, "Cs-137", 9.0)];
    let verdict = build_verdict(&identifications, &[], &nuclides);
    assert_eq!(verdict.kind, VerdictKind::Nuclide);
    assert_eq!(verdict.nuclide_id, Some(cs137));
    assert_eq!(verdict.confidence, Confidence::High);
}

#[test]
fn chain_with_single_explained_peak_falls_back_to_nuclide() {
    let pb212 = NuclideId::new(82, 130, 0);
    let identifications = vec![identification(238.6, 100.0, pb212, "Pb-212")];
    let members = vec![evidence(pb212, "Pb-212", 5.0)];
    let chains = vec![thorium_chain(members.clone())];
    let verdict = build_verdict(&identifications, &chains, &members);
    assert_eq!(verdict.kind, VerdictKind::Nuclide);
    assert_eq!(verdict.nuclide_id, Some(pb212));
}
