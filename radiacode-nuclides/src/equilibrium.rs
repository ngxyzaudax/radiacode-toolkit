use std::collections::{HashMap, VecDeque};

use crate::model::NuclideId;
use crate::series::ChainSeries;
use crate::topology::{has_emissions, topology_decays, topology_half_life_secs};

#[derive(Debug, Clone, PartialEq)]
pub struct MemberWeight {
    pub id: NuclideId,
    pub depth: usize,
    pub weight: f64,
    pub half_life_secs: Option<f64>,
    pub has_emissions: bool,
}

pub fn equilibrium_weights(series: &ChainSeries) -> Vec<MemberWeight> {
    let mut weights: HashMap<NuclideId, f64> = HashMap::from([(series.head, 1.0)]);
    let mut depths: HashMap<NuclideId, usize> = HashMap::from([(series.head, 0)]);
    let mut queue = VecDeque::from([series.head]);
    while let Some(id) = queue.pop_front() {
        let from_weight = weights.get(&id).copied().unwrap_or(0.0);
        let from_depth = depths.get(&id).copied().unwrap_or(0);
        for branch in topology_decays(id) {
            let share = from_weight * branch.branching_pct / 100.0;
            *weights.entry(branch.daughter).or_insert(0.0) += share;
            depths
                .entry(branch.daughter)
                .and_modify(|depth| *depth = (*depth).min(from_depth + 1))
                .or_insert(from_depth + 1);
            queue.push_back(branch.daughter);
        }
    }
    let mut members = series
        .members
        .iter()
        .copied()
        .filter(|id| weights.contains_key(id))
        .map(|id| MemberWeight {
            id,
            depth: depths.get(&id).copied().unwrap_or(0),
            weight: weights.get(&id).copied().unwrap_or(0.0),
            half_life_secs: topology_half_life_secs(id),
            has_emissions: has_emissions(id),
        })
        .collect::<Vec<_>>();
    members.sort_by(|left, right| {
        left.depth
            .cmp(&right.depth)
            .then_with(|| left.id.cmp(&right.id))
    });
    members
}

pub fn bottleneck_member(weights: &[MemberWeight], head: NuclideId) -> Option<&MemberWeight> {
    weights
        .iter()
        .filter(|member| member.id != head)
        .filter(|member| member.half_life_secs.is_some())
        .max_by(|left, right| {
            left.half_life_secs
                .partial_cmp(&right.half_life_secs)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
}

pub fn time_to_equilibrium_secs(weights: &[MemberWeight], head: NuclideId) -> Option<f64> {
    bottleneck_member(weights, head).and_then(|member| member.half_life_secs.map(|secs| secs * 7.0))
}
