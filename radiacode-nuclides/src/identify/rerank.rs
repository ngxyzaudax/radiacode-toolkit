use crate::match_peaks::{NuclideMatch, PeakIdentification};
use crate::model::NuclideId;
use crate::series::chain_series_by_head;
use crate::source_summary::SourceSummary;

const CHAIN_MEMBER_BONUS: f64 = 0.35;
const WINNING_NUCLIDE_BONUS: f64 = 0.2;

pub fn rerank_for_chain_head(
    identifications: Vec<PeakIdentification>,
    chain_head: crate::model::NuclideId,
) -> Vec<PeakIdentification> {
    let chain_members = chain_series_by_head(chain_head).map(|series| series.members.as_slice());
    identifications
        .into_iter()
        .map(|mut identification| {
            identification.candidates = rerank_peak_candidates(
                std::mem::take(&mut identification.candidates),
                chain_members,
                None,
            );
            identification
        })
        .collect()
}

pub fn rerank_identifications(
    mut identifications: Vec<PeakIdentification>,
    sources: &SourceSummary,
) -> Vec<PeakIdentification> {
    let preferred_chain = sources.verdict.chain_head.and_then(chain_series_by_head);
    let preferred_nuclide = sources.verdict.nuclide_id;
    for identification in &mut identifications {
        identification.candidates = rerank_peak_candidates(
            std::mem::take(&mut identification.candidates),
            preferred_chain.map(|series| series.members.as_slice()),
            preferred_nuclide,
        );
    }
    identifications
}

fn rerank_peak_candidates(
    mut candidates: Vec<NuclideMatch>,
    chain_members: Option<&[NuclideId]>,
    preferred_nuclide: Option<NuclideId>,
) -> Vec<NuclideMatch> {
    for candidate in &mut candidates {
        if chain_members.is_some_and(|members| members.contains(&candidate.nuclide_id)) {
            candidate.score *= 1.0 + CHAIN_MEMBER_BONUS;
        }
        if preferred_nuclide == Some(candidate.nuclide_id) {
            candidate.score *= 1.0 + WINNING_NUCLIDE_BONUS;
        }
    }
    super::candidates::rank_candidates(candidates)
}
