use radiacode_nuclides::NuclideId;

use super::LevelIndex;
use super::build::upsert_topology_candidate;
use super::bundle::gammas_for_level;
use super::isomer::{daughter_isomer_candidates_from_bundle, isomer_candidates_from_bundle};
use super::level_index::register_level;
use super::test_fixtures::{bundle, candidate, parent_row, row};

#[test]
fn isomer_candidates_register_levels_and_collect_gammas() {
    let gamma = parent_row(&[
        ("decay", "IT"),
        ("energy", "140.511"),
        ("intensity", "89"),
        ("d_z", "43"),
        ("d_n", "56"),
    ]);
    let radiation = bundle(vec![gamma], Vec::new());
    let mut index = LevelIndex::new();
    let isomers = isomer_candidates_from_bundle(&radiation, &mut index);
    assert_eq!(isomers.len(), 1);
    assert_eq!(isomers[0].id, NuclideId::new(43, 56, 1));
    assert_eq!(isomers[0].half_life_secs, Some(21624.0));
    assert_eq!(isomers[0].decays.len(), 1);
    assert_eq!(gammas_for_level(&radiation, 142.683).len(), 1);
}

#[test]
fn daughter_isomers_require_known_levels() {
    let beta = row(&[
        ("p_energy", "0"),
        ("decay", "B-"),
        ("d_z", "43"),
        ("d_n", "56"),
        ("d_symbol", "Tc"),
        ("daughter_level_energy", "142.683"),
    ]);
    let radiation = bundle(Vec::new(), vec![beta.clone(), beta]);
    assert!(daughter_isomer_candidates_from_bundle(&radiation, &LevelIndex::new()).is_empty());
    let mut index = LevelIndex::new();
    register_level(&mut index, 43, 56, 142.683);
    let daughters = daughter_isomer_candidates_from_bundle(&radiation, &index);
    assert_eq!(daughters.len(), 1);
    assert!(daughters[0].force_include);
}

#[test]
fn upsert_topology_prefers_known_level_data() {
    let id = NuclideId::new(43, 56, 1);
    let mut candidates = vec![candidate(id, 0.0)];
    let mut update = candidate(id, 142.683);
    update.half_life_secs = Some(21624.0);
    update.half_life_text = "6.01 h".into();
    upsert_topology_candidate(&mut candidates, update);
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].half_life_secs, Some(21624.0));
    upsert_topology_candidate(&mut candidates, candidate(NuclideId::new(1, 0, 0), 0.0));
    assert_eq!(candidates.len(), 2);
}
