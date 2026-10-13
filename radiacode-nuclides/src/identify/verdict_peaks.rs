use crate::identify::verdict::Confidence;
use crate::match_peaks::PeakIdentification;
use crate::model::NuclideId;
use crate::source_summary::NuclideEvidence;

const HIGH_CONFIDENCE_FRACTION: f64 = 0.6;
const MEDIUM_CONFIDENCE_FRACTION: f64 = 0.35;

fn top_candidate_is(identification: &PeakIdentification, id: NuclideId) -> bool {
    identification
        .candidates
        .first()
        .is_some_and(|candidate| candidate.nuclide_id == id)
}

pub fn peak_strength_for_nuclide(identifications: &[PeakIdentification], id: NuclideId) -> f64 {
    identifications
        .iter()
        .filter(|identification| top_candidate_is(identification, id))
        .map(|identification| identification.peak.counts)
        .sum()
}

pub fn explained_for_nuclide(identifications: &[PeakIdentification], id: NuclideId) -> u32 {
    identifications
        .iter()
        .filter(|identification| top_candidate_is(identification, id))
        .count() as u32
}

pub fn explained_peak_count(
    identifications: &[PeakIdentification],
    members: &[NuclideEvidence],
) -> u32 {
    identifications
        .iter()
        .filter(|identification| {
            members
                .iter()
                .any(|member| top_candidate_is(identification, member.id))
        })
        .count() as u32
}

pub fn confidence_from_fraction(explained: u32, total: u32) -> Confidence {
    if total == 0 {
        return Confidence::Low;
    }
    let fraction = explained as f64 / total as f64;
    if fraction >= HIGH_CONFIDENCE_FRACTION {
        Confidence::High
    } else if fraction >= MEDIUM_CONFIDENCE_FRACTION {
        Confidence::Medium
    } else {
        Confidence::Low
    }
}
