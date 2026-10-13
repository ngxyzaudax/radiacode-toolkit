use std::collections::HashMap;

use crate::identify::SourceVerdict;
use crate::match_peaks::PeakIdentification;
use crate::model::NuclideId;

#[derive(Debug, Clone, PartialEq)]
pub struct NuclideEvidence {
    pub id: NuclideId,
    pub display_name: String,
    pub matched_lines: u32,
    pub score: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChainEvidence {
    pub head: NuclideId,
    pub name: String,
    pub family: String,
    pub matched_members: u32,
    pub score: f64,
    pub members: Vec<NuclideEvidence>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SourceSummary {
    pub nuclides: Vec<NuclideEvidence>,
    pub chains: Vec<ChainEvidence>,
    pub verdict: SourceVerdict,
}

impl SourceSummary {
    pub fn sort_evidence(&mut self) {
        self.nuclides.sort_by(|left, right| {
            right
                .score
                .partial_cmp(&left.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| left.display_name.cmp(&right.display_name))
        });
        self.chains.sort_by(|left, right| {
            right
                .score
                .partial_cmp(&left.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| right.matched_members.cmp(&left.matched_members))
                .then_with(|| left.name.cmp(&right.name))
        });
    }
}

pub fn collect_nuclide_evidence(
    identifications: &[PeakIdentification],
    max_candidates: usize,
) -> Vec<NuclideEvidence> {
    let mut nuclide_totals: HashMap<NuclideId, NuclideEvidence> = HashMap::new();
    for identification in identifications {
        for (rank, candidate) in identification
            .candidates
            .iter()
            .take(max_candidates)
            .enumerate()
        {
            let weight = 1.0 / (rank as f64 + 1.0);
            nuclide_totals
                .entry(candidate.nuclide_id)
                .and_modify(|entry| {
                    entry.matched_lines += 1;
                    entry.score += candidate.score * weight;
                })
                .or_insert(NuclideEvidence {
                    id: candidate.nuclide_id,
                    display_name: candidate.display_name.clone(),
                    matched_lines: 1,
                    score: candidate.score * weight,
                });
        }
    }
    nuclide_totals.into_values().collect()
}
