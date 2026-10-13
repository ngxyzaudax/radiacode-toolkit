use std::collections::HashMap;

use crate::equilibrium::equilibrium_weights;
use crate::identify::completeness::nuclide_hypothesis_score;
use crate::identify::window::AnalysisWindow;
use crate::match_peaks::{MatchParams, PeakIdentification};
use crate::model::NuclideId;
use crate::series::{chain_series_by_head, is_natural_series_head, series_for_member};
use crate::source_summary::{ChainEvidence, NuclideEvidence};

const MEMBER_COHERENCE_BONUS: f64 = 0.35;
const NATURAL_SERIES_BOOST: f64 = 1.6;

pub fn score_chains(
    identifications: &[PeakIdentification],
    window: AnalysisWindow,
    params: MatchParams,
    nuclides: &[NuclideEvidence],
) -> Vec<ChainEvidence> {
    let mut grouped: HashMap<NuclideId, ChainEvidence> = HashMap::new();
    for nuclide in nuclides {
        let Some(series) = series_for_member(nuclide.id) else {
            continue;
        };
        let member_score = nuclide_hypothesis_score(nuclide.id, identifications, window, params);
        if member_score <= 0.0 {
            continue;
        }
        grouped
            .entry(series.head)
            .and_modify(|entry| {
                if !entry.members.iter().any(|member| member.id == nuclide.id) {
                    entry.matched_members += 1;
                    entry.score += member_score;
                    entry.members.push(nuclide.clone());
                }
            })
            .or_insert_with(|| ChainEvidence {
                head: series.head,
                name: series.name.clone(),
                family: series.family.clone(),
                matched_members: 1,
                score: member_score,
                members: vec![nuclide.clone()],
            });
    }
    let mut chains: Vec<ChainEvidence> = grouped
        .into_values()
        .map(|mut chain| {
            apply_coherence_bonus(&mut chain, identifications, window, params);
            chain
        })
        .collect();
    for chain in &mut chains {
        chain.members.sort_by(|left, right| {
            right
                .score
                .partial_cmp(&left.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }
    chains.sort_by(|left, right| {
        chain_rank_tier(right)
            .cmp(&chain_rank_tier(left))
            .then_with(|| {
                right
                    .score
                    .partial_cmp(&left.score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .then_with(|| right.matched_members.cmp(&left.matched_members))
            .then_with(|| left.name.cmp(&right.name))
    });
    chains
}

fn chain_rank_tier(chain: &ChainEvidence) -> u8 {
    if is_natural_series_head(chain.head) && chain.matched_members >= 2 {
        return 3;
    }
    if chain.matched_members >= 2 {
        return 2;
    }
    if is_natural_series_head(chain.head) {
        return 1;
    }
    0
}

fn apply_coherence_bonus(
    chain: &mut ChainEvidence,
    identifications: &[PeakIdentification],
    window: AnalysisWindow,
    params: MatchParams,
) {
    let Some(series) = chain_series_by_head(chain.head) else {
        return;
    };
    let weights = equilibrium_weights(series);
    let weight_by_id: HashMap<NuclideId, f64> = weights
        .iter()
        .map(|member| (member.id, member.weight))
        .collect();
    let mut weighted = 0.0_f64;
    for member in &chain.members {
        let equilibrium = weight_by_id.get(&member.id).copied().unwrap_or(1.0);
        let hypothesis = nuclide_hypothesis_score(member.id, identifications, window, params);
        weighted += hypothesis * equilibrium.sqrt();
    }
    let member_factor =
        1.0 + MEMBER_COHERENCE_BONUS * (chain.matched_members.saturating_sub(1) as f64);
    chain.score = weighted * member_factor;
    if is_natural_series_head(chain.head) {
        chain.score *= NATURAL_SERIES_BOOST;
    }
}
