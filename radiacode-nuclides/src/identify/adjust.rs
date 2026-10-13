use crate::identify::candidates::rank_candidates;
use crate::identify::completeness::CompletenessScore;
use crate::identify::completeness::completeness_for_nuclide;
use crate::identify::window::AnalysisWindow;
use crate::match_peaks::{MatchParams, PeakIdentification};

pub fn apply_completeness_adjustments(
    mut identifications: Vec<PeakIdentification>,
    window: AnalysisWindow,
    params: MatchParams,
) -> Vec<PeakIdentification> {
    let snapshot = identifications.clone();
    for identification in &mut identifications {
        for candidate in &mut identification.candidates {
            let completeness =
                completeness_for_nuclide(candidate.nuclide_id, &snapshot, window, params);
            candidate.score = adjusted_candidate_score(candidate.score, &completeness);
        }
        identification.candidates = rank_candidates(std::mem::take(&mut identification.candidates));
    }
    identifications
}

fn adjusted_candidate_score(raw_score: f64, completeness: &CompletenessScore) -> f64 {
    let total = completeness.explained + completeness.missing;
    let ratio = if total > 0 {
        completeness.explained as f64 / total as f64
    } else {
        1.0
    };
    ((raw_score - completeness.penalty) * ratio).max(0.0)
}
