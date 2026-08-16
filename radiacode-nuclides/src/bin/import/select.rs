use std::collections::{HashMap, HashSet, VecDeque};

use radiacode_nuclides::NuclideId;

use super::Candidate;
use super::MIN_HALF_LIFE_SECS;

pub fn select_nuclides(candidates: &[Candidate]) -> Vec<Candidate> {
    let mut selected = candidates
        .iter()
        .filter(|candidate| {
            candidate.force_include
                || candidate.half_life_secs.is_none()
                || candidate.half_life_secs.unwrap_or(0.0) >= MIN_HALF_LIFE_SECS
        })
        .cloned()
        .collect::<Vec<_>>();
    selected.sort_by(|left, right| fetch_rank(left).cmp(&fetch_rank(right)));
    selected
}

fn fetch_rank(candidate: &Candidate) -> u8 {
    if candidate.force_include {
        return 0;
    }
    if candidate.half_life_secs.is_none() {
        return 2;
    }
    1
}

pub fn force_chain_members(candidates: &mut [Candidate]) {
    let forced = chain_closure_ids(candidates);
    for candidate in candidates.iter_mut() {
        if forced.contains(&candidate.id) {
            candidate.force_include = true;
        }
    }
}

pub fn chain_closure_ids(candidates: &[Candidate]) -> HashSet<NuclideId> {
    let by_id: HashMap<NuclideId, &Candidate> = candidates
        .iter()
        .map(|candidate| (candidate.id, candidate))
        .collect();
    let seeds = chain_seed_ids();
    let mut forced = HashSet::new();
    let mut queue: VecDeque<NuclideId> = seeds.into_iter().collect();
    while let Some(id) = queue.pop_front() {
        if !forced.insert(id) {
            continue;
        }
        let Some(candidate) = by_id.get(&id) else {
            continue;
        };
        for branch in &candidate.decays {
            queue.push_back(branch.daughter);
        }
    }
    forced
}

fn chain_seed_ids() -> HashSet<NuclideId> {
    [
        (92, 146),
        (92, 143),
        (90, 142),
        (93, 144),
        (55, 82),
        (19, 21),
        (27, 33),
        (95, 146),
    ]
    .into_iter()
    .map(|(z, n)| NuclideId::new(z, n, 0))
    .collect()
}
