use crate::identify::verdict_peaks::{confidence_from_fraction, explained_peak_count};
use crate::identify::verdict_select::{chosen_nuclide_for_verdict, select_chain_verdict};
use crate::match_peaks::PeakIdentification;
use crate::model::NuclideId;
use crate::series::is_natural_series_head;
use crate::source_summary::{ChainEvidence, NuclideEvidence};

const MIN_CHAIN_EXPLAINED_PEAKS: u32 = 2;
const MIN_ARTIFICIAL_CHAIN_FRACTION: f64 = 0.35;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerdictKind {
    Chain,
    Nuclide,
    Inconclusive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Confidence {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceVerdict {
    pub kind: VerdictKind,
    pub headline: String,
    pub confidence: Confidence,
    pub explained_peaks: u32,
    pub total_peaks: u32,
    pub chain_head: Option<NuclideId>,
    pub nuclide_id: Option<NuclideId>,
}

impl Default for SourceVerdict {
    fn default() -> Self {
        inconclusive_verdict("No confident source match", 0)
    }
}

pub fn build_verdict(
    identifications: &[PeakIdentification],
    chains: &[ChainEvidence],
    nuclides: &[NuclideEvidence],
) -> SourceVerdict {
    let total_peaks = identifications.len() as u32;
    if total_peaks == 0 {
        return SourceVerdict::default();
    }
    select_chain_verdict(chains)
        .and_then(|chain| chain_verdict(identifications, chain, total_peaks))
        .or_else(|| {
            chosen_nuclide_for_verdict(identifications, nuclides)
                .and_then(|member| nuclide_verdict(identifications, &member, total_peaks))
        })
        .unwrap_or_else(|| inconclusive_verdict("Mixed or inconclusive source", total_peaks))
}

fn chain_verdict(
    identifications: &[PeakIdentification],
    chain: &ChainEvidence,
    total_peaks: u32,
) -> Option<SourceVerdict> {
    let explained = explained_peak_count(identifications, &chain.members);
    if !chain_qualifies(chain.head, explained, total_peaks) {
        return None;
    }
    let confidence = confidence_from_fraction(explained, total_peaks);
    Some(SourceVerdict {
        kind: VerdictKind::Chain,
        headline: headline(&chain.name, explained, total_peaks, confidence),
        confidence,
        explained_peaks: explained,
        total_peaks,
        chain_head: Some(chain.head),
        nuclide_id: None,
    })
}

fn chain_qualifies(head: NuclideId, explained: u32, total_peaks: u32) -> bool {
    let fraction = explained as f64 / total_peaks as f64;
    explained >= MIN_CHAIN_EXPLAINED_PEAKS
        && (is_natural_series_head(head) || fraction >= MIN_ARTIFICIAL_CHAIN_FRACTION)
}

fn nuclide_verdict(
    identifications: &[PeakIdentification],
    member: &NuclideEvidence,
    total_peaks: u32,
) -> Option<SourceVerdict> {
    let explained = explained_peak_count(identifications, std::slice::from_ref(member));
    if explained == 0 {
        return None;
    }
    let confidence = confidence_from_fraction(explained, total_peaks);
    Some(SourceVerdict {
        kind: VerdictKind::Nuclide,
        headline: headline(&member.display_name, explained, total_peaks, confidence),
        confidence,
        explained_peaks: explained,
        total_peaks,
        chain_head: None,
        nuclide_id: Some(member.id),
    })
}

fn inconclusive_verdict(headline: &str, total_peaks: u32) -> SourceVerdict {
    SourceVerdict {
        kind: VerdictKind::Inconclusive,
        headline: headline.into(),
        confidence: Confidence::Low,
        explained_peaks: 0,
        total_peaks,
        chain_head: None,
        nuclide_id: None,
    }
}

fn headline(name: &str, explained: u32, total_peaks: u32, confidence: Confidence) -> String {
    format!(
        "{name} - {explained} of {total_peaks} peaks explained - {} confidence",
        confidence_label(confidence)
    )
}

fn confidence_label(confidence: Confidence) -> &'static str {
    match confidence {
        Confidence::High => "high",
        Confidence::Medium => "medium",
        Confidence::Low => "low",
    }
}
