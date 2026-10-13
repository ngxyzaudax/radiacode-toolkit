use crate::chain_lines::chain_lines;
use crate::equilibrium::MemberWeight;
use crate::identify::lead_chain::leading_chain_for_rerank;
use crate::model::NuclideId;
use crate::source_summary::ChainEvidence;

fn member(id: NuclideId, weight: f64) -> MemberWeight {
    MemberWeight {
        id,
        depth: 0,
        weight,
        half_life_secs: None,
        has_emissions: true,
    }
}

fn cs137_id() -> NuclideId {
    NuclideId::new(55, 82, 0)
}

fn ba137m_id() -> NuclideId {
    NuclideId::new(56, 81, 1)
}

fn evidence(head: NuclideId, matched_members: u32, score: f64) -> ChainEvidence {
    ChainEvidence {
        head,
        name: String::new(),
        family: String::new(),
        matched_members,
        score,
        members: Vec::new(),
    }
}

#[test]
fn unknown_members_are_skipped() {
    let weights = [member(NuclideId::new(1, 200, 0), 1.0)];
    assert!(chain_lines(&weights).is_empty());
}

#[test]
fn lines_are_sorted_by_scaled_intensity_and_scaled_by_weight() {
    let weights = [member(ba137m_id(), 0.5)];
    let lines = chain_lines(&weights);
    assert!(!lines.is_empty());
    let strongest = &lines[0];
    assert!((strongest.line.energy_kev - 661.7).abs() < 1.0);
    assert!((strongest.scaled_intensity_pct - strongest.line.intensity_pct * 0.5).abs() < 1e-9);
    assert!(
        lines
            .windows(2)
            .all(|pair| pair[0].scaled_intensity_pct >= pair[1].scaled_intensity_pct)
    );
}

#[test]
fn leading_chain_prefers_natural_series_with_two_or_more_members() {
    let u238 = NuclideId::new(92, 146, 0);
    let chains = [
        evidence(cs137_id(), 3, 9.0),
        evidence(u238, 2, 4.0),
        evidence(NuclideId::new(90, 142, 0), 1, 8.0),
    ];
    let leading = leading_chain_for_rerank(&chains).expect("chain");
    assert_eq!(leading.head, u238);
}

#[test]
fn leading_chain_falls_back_to_best_scoring_non_natural_chain() {
    let chains = [
        evidence(cs137_id(), 2, 3.0),
        evidence(ba137m_id(), 2, 5.0),
        evidence(NuclideId::new(92, 146, 0), 1, 9.0),
    ];
    let leading = leading_chain_for_rerank(&chains).expect("chain");
    assert_eq!(leading.head, ba137m_id());
    assert!(leading_chain_for_rerank(&[evidence(cs137_id(), 1, 9.0)]).is_none());
}
