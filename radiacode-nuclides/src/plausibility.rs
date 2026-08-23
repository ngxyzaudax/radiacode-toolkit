use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::OnceLock;

use crate::model::NuclideId;
use crate::topology::{decay_catalog, topology_half_life_secs, topology_parents};

static SUPPORTED_HALF_LIFE: OnceLock<HashMap<NuclideId, f64>> = OnceLock::new();

pub fn supported_half_life_secs(id: NuclideId) -> Option<f64> {
    supported_map().get(&id).copied()
}

pub fn is_plausible(id: NuclideId, min_secs: f64) -> bool {
    supported_half_life_secs(id).is_some_and(|secs| secs >= min_secs)
}

fn supported_map() -> &'static HashMap<NuclideId, f64> {
    SUPPORTED_HALF_LIFE.get_or_init(build_supported_map)
}

fn build_supported_map() -> HashMap<NuclideId, f64> {
    let mut supported: HashMap<NuclideId, f64> = HashMap::new();
    for entry in &decay_catalog().entries {
        let own = topology_half_life_secs(entry.id).unwrap_or(0.0);
        supported.insert(entry.id, own);
    }
    let ids: Vec<NuclideId> = supported.keys().copied().collect();
    for id in ids {
        let value = upstream_supported_half_life(id);
        supported.insert(id, value);
    }
    supported
}

fn upstream_supported_half_life(start: NuclideId) -> f64 {
    let mut best = topology_half_life_secs(start).unwrap_or(0.0);
    let mut seen = HashSet::from([start]);
    let mut queue = VecDeque::from([start]);
    while let Some(id) = queue.pop_front() {
        for parent in topology_parents(id) {
            if !seen.insert(*parent) {
                continue;
            }
            queue.push_back(*parent);
            best = best.max(topology_half_life_secs(*parent).unwrap_or(0.0));
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::{is_plausible, supported_half_life_secs};
    use crate::model::NuclideId;

    #[test]
    fn thorium_daughters_inherit_long_lived_support() {
        let tl208 = NuclideId::new(81, 127, 0);
        let po212 = NuclideId::new(84, 128, 0);
        assert!(supported_half_life_secs(tl208).unwrap_or(0.0) > 1.0e15);
        assert!(supported_half_life_secs(po212).unwrap_or(0.0) > 1.0e15);
    }

    #[test]
    fn cs137_supports_ba137m() {
        let ba137m = NuclideId::new(56, 81, 1);
        assert!(supported_half_life_secs(ba137m).unwrap_or(0.0) > 1.0e8);
    }

    #[test]
    fn exotic_isomers_fail_plausibility_gate() {
        let dy146m = NuclideId::new(66, 80, 1);
        let zn78m = NuclideId::new(30, 48, 1);
        assert!(!is_plausible(dy146m, 600.0));
        assert!(!is_plausible(zn78m, 600.0));
    }
}
