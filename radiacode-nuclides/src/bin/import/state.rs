use std::collections::HashSet;

use radiacode_nuclides::{Nuclide, NuclideId};

use super::{Candidate, LevelIndex};

#[derive(Default)]
pub struct ImportState {
    pub nuclides: Vec<Nuclide>,
    pub topology_candidates: Vec<Candidate>,
    pub level_index: LevelIndex,
    pub seen_ids: HashSet<NuclideId>,
    pub pending_isomers: Vec<Candidate>,
}

impl ImportState {
    pub fn reached_limit(&self, limit: Option<usize>) -> bool {
        limit.is_some_and(|cap| self.nuclides.len() >= cap)
    }
}
