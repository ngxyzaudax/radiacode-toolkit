use crate::identify::window::AnalysisWindow;
use crate::match_peaks::{NuclideMatch, PeakIdentification};
use crate::model::NuclideId;
use crate::series::is_natural_series_head;
use crate::source_summary::{ChainEvidence, NuclideEvidence};

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
        Self {
            kind: VerdictKind::Inconclusive,
            headline: "No confident source match".into(),
            confidence: Confidence::Low,
            explained_peaks: 0,
            total_peaks: 0,
            chain_head: None,
            nuclide_id: None,
        }
    }
}

pub fn build_verdict(
    identifications: &[PeakIdentification],
    _window: AnalysisWindow,
    chains: &[ChainEvidence],
    nuclides: &[NuclideEvidence],
) -> SourceVerdict {
    let total_peaks = identifications.len() as u32;
    if total_peaks == 0 {
        return SourceVerdict::default();
    }
    if let Some(chain) = select_chain_verdict(chains) {
        let explained = explained_peak_count(identifications, &chain.members);
        let fraction = explained as f64 / total_peaks as f64;
        let chain_qualifies =
            explained >= 2 && (is_natural_series_head(chain.head) || fraction >= 0.35);
        if chain_qualifies {
            let confidence = confidence_from_fraction(explained, total_peaks);
            return SourceVerdict {
                kind: VerdictKind::Chain,
                headline: format!(
                    "{} - {} of {} peaks explained - {} confidence",
                    chain.name,
                    explained,
                    total_peaks,
                    confidence_label(confidence)
                ),
                confidence,
                explained_peaks: explained,
                total_peaks,
                chain_head: Some(chain.head),
                nuclide_id: None,
            };
        }
    }
    if let Some(member) = chosen_nuclide_for_verdict(identifications, nuclides) {
        let explained = explained_peak_count(identifications, std::slice::from_ref(&member));
        if explained > 0 {
            let confidence = confidence_from_fraction(explained, total_peaks);
            return SourceVerdict {
                kind: VerdictKind::Nuclide,
                headline: format!(
                    "{} - {} of {} peaks explained - {} confidence",
                    member.display_name,
                    explained,
                    total_peaks,
                    confidence_label(confidence)
                ),
                confidence,
                explained_peaks: explained,
                total_peaks,
                chain_head: None,
                nuclide_id: Some(member.id),
            };
        }
    }
    SourceVerdict {
        kind: VerdictKind::Inconclusive,
        headline: "Mixed or inconclusive source".into(),
        confidence: Confidence::Low,
        explained_peaks: 0,
        total_peaks,
        chain_head: None,
        nuclide_id: None,
    }
}

fn select_chain_verdict(chains: &[ChainEvidence]) -> Option<&ChainEvidence> {
    chains
        .iter()
        .filter(|chain| is_natural_series_head(chain.head) && chain.matched_members >= 2)
        .max_by(|left, right| {
            left.score
                .partial_cmp(&right.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| right.matched_members.cmp(&left.matched_members))
        })
        .or_else(|| {
            chains
                .iter()
                .filter(|chain| chain.matched_members >= 2)
                .max_by(|left, right| {
                    left.score
                        .partial_cmp(&right.score)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
        })
}

fn chosen_nuclide_for_verdict(
    identifications: &[PeakIdentification],
    nuclides: &[NuclideEvidence],
) -> Option<NuclideEvidence> {
    let ranked = select_nuclide_verdict(identifications, nuclides).cloned();
    let strongest = strongest_peak_nuclide_match(identifications).map(nuclide_evidence_from_match);
    let high_energy = dominant_high_energy_nuclide(identifications);
    [high_energy, strongest, ranked]
        .into_iter()
        .flatten()
        .max_by(|left, right| {
            peak_strength_for_nuclide(identifications, left.id)
                .partial_cmp(&peak_strength_for_nuclide(identifications, right.id))
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| {
                    left.score
                        .partial_cmp(&right.score)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
        })
}

fn dominant_high_energy_nuclide(
    identifications: &[PeakIdentification],
) -> Option<NuclideEvidence> {
    const HIGH_ENERGY_KEV: f64 = 1200.0;
    identifications
        .iter()
        .filter(|identification| identification.peak.energy_kev >= HIGH_ENERGY_KEV)
        .max_by(|left, right| {
            left.peak
                .counts
                .partial_cmp(&right.peak.counts)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .and_then(|identification| identification.candidates.first())
        .map(nuclide_evidence_from_match)
}

fn nuclide_evidence_from_match(candidate: &NuclideMatch) -> NuclideEvidence {
    NuclideEvidence {
        id: candidate.nuclide_id,
        display_name: candidate.display_name.clone(),
        matched_lines: 1,
        score: candidate.score,
    }
}

fn strongest_peak_nuclide_match(
    identifications: &[PeakIdentification],
) -> Option<&NuclideMatch> {
    identifications
        .iter()
        .max_by(|left, right| {
            left.peak
                .counts
                .partial_cmp(&right.peak.counts)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .and_then(|identification| identification.candidates.first())
}

fn select_nuclide_verdict<'a>(
    identifications: &'a [PeakIdentification],
    nuclides: &'a [NuclideEvidence],
) -> Option<&'a NuclideEvidence> {
    nuclides
        .iter()
        .filter(|nuclide| nuclide.score > 0.0)
        .max_by(|left, right| {
            let left_explained = explained_for_nuclide(identifications, left.id);
            let right_explained = explained_for_nuclide(identifications, right.id);
            left_explained
                .cmp(&right_explained)
                .then_with(|| {
                    peak_strength_for_nuclide(identifications, left.id)
                        .partial_cmp(&peak_strength_for_nuclide(identifications, right.id))
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .then_with(|| {
                    left.score
                        .partial_cmp(&right.score)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
        })
}

fn peak_strength_for_nuclide(
    identifications: &[PeakIdentification],
    id: NuclideId,
) -> f64 {
    identifications
        .iter()
        .filter(|identification| {
            identification
                .candidates
                .first()
                .is_some_and(|candidate| candidate.nuclide_id == id)
        })
        .map(|identification| identification.peak.counts)
        .sum()
}

fn explained_for_nuclide(identifications: &[PeakIdentification], id: NuclideId) -> u32 {
    identifications
        .iter()
        .filter(|identification| {
            identification
                .candidates
                .first()
                .is_some_and(|candidate| candidate.nuclide_id == id)
        })
        .count() as u32
}

fn explained_peak_count(
    identifications: &[PeakIdentification],
    members: &[NuclideEvidence],
) -> u32 {
    identifications
        .iter()
        .filter(|identification| {
            identification.candidates.first().is_some_and(|candidate| {
                members
                    .iter()
                    .any(|member| member.id == candidate.nuclide_id)
            })
        })
        .count() as u32
}

fn confidence_from_fraction(explained: u32, total: u32) -> Confidence {
    if total == 0 {
        return Confidence::Low;
    }
    let fraction = explained as f64 / total as f64;
    if fraction >= 0.6 {
        Confidence::High
    } else if fraction >= 0.35 {
        Confidence::Medium
    } else {
        Confidence::Low
    }
}

fn confidence_label(confidence: Confidence) -> &'static str {
    match confidence {
        Confidence::High => "high",
        Confidence::Medium => "medium",
        Confidence::Low => "low",
    }
}

#[cfg(test)]
mod tests {
    use super::confidence_from_fraction;
    use super::Confidence;

    #[test]
    fn confidence_thresholds() {
        assert_eq!(confidence_from_fraction(4, 5), Confidence::High);
        assert_eq!(confidence_from_fraction(2, 5), Confidence::Medium);
        assert_eq!(confidence_from_fraction(1, 5), Confidence::Low);
    }
}
