use std::cmp::Ordering;

use crate::identify::verdict_peaks::{explained_for_nuclide, peak_strength_for_nuclide};
use crate::match_peaks::{NuclideMatch, PeakIdentification};
use crate::series::is_natural_series_head;
use crate::source_summary::{ChainEvidence, NuclideEvidence};

const HIGH_ENERGY_KEV: f64 = 1200.0;
const MIN_CHAIN_MEMBERS: u32 = 2;

pub fn select_chain_verdict(chains: &[ChainEvidence]) -> Option<&ChainEvidence> {
    let qualifying = chains
        .iter()
        .filter(|chain| chain.matched_members >= MIN_CHAIN_MEMBERS);
    qualifying
        .clone()
        .filter(|chain| is_natural_series_head(chain.head))
        .max_by(compare_chain_evidence)
        .or_else(|| qualifying.max_by(|left, right| left.score.total_cmp(&right.score)))
}

fn compare_chain_evidence(left: &&ChainEvidence, right: &&ChainEvidence) -> Ordering {
    left.score
        .total_cmp(&right.score)
        .then_with(|| right.matched_members.cmp(&left.matched_members))
}

pub fn chosen_nuclide_for_verdict(
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
                .total_cmp(&peak_strength_for_nuclide(identifications, right.id))
                .then_with(|| left.score.total_cmp(&right.score))
        })
}

fn dominant_high_energy_nuclide(identifications: &[PeakIdentification]) -> Option<NuclideEvidence> {
    identifications
        .iter()
        .filter(|identification| identification.peak.energy_kev >= HIGH_ENERGY_KEV)
        .max_by(compare_peak_counts)
        .and_then(|identification| identification.candidates.first())
        .map(nuclide_evidence_from_match)
}

fn strongest_peak_nuclide_match(identifications: &[PeakIdentification]) -> Option<&NuclideMatch> {
    identifications
        .iter()
        .max_by(compare_peak_counts)
        .and_then(|identification| identification.candidates.first())
}

fn compare_peak_counts(left: &&PeakIdentification, right: &&PeakIdentification) -> Ordering {
    left.peak.counts.total_cmp(&right.peak.counts)
}

fn nuclide_evidence_from_match(candidate: &NuclideMatch) -> NuclideEvidence {
    NuclideEvidence {
        id: candidate.nuclide_id,
        display_name: candidate.display_name.clone(),
        matched_lines: 1,
        score: candidate.score,
    }
}

fn select_nuclide_verdict<'a>(
    identifications: &'a [PeakIdentification],
    nuclides: &'a [NuclideEvidence],
) -> Option<&'a NuclideEvidence> {
    nuclides
        .iter()
        .filter(|nuclide| nuclide.score > 0.0)
        .max_by(|left, right| {
            explained_for_nuclide(identifications, left.id)
                .cmp(&explained_for_nuclide(identifications, right.id))
                .then_with(|| {
                    peak_strength_for_nuclide(identifications, left.id)
                        .total_cmp(&peak_strength_for_nuclide(identifications, right.id))
                })
                .then_with(|| left.score.total_cmp(&right.score))
        })
}
