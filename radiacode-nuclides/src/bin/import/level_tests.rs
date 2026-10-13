use std::collections::HashMap;

use radiacode_nuclides::{DecayBranch, DecayMode, NuclideId};

use super::LevelIndex;
use super::branches::{level_branches_from_bundle, merge_decay_branches};
use super::level_index::{
    build_level_index_from_bundles, merge_levels_from_bundles, metastable_for_energy,
    register_level,
};
use super::test_fixtures::{bundle, parent_row};

#[test]
fn register_level_orders_isomers_by_energy() {
    let mut index = LevelIndex::new();
    assert_eq!(register_level(&mut index, 72, 106, 2446.0), 1);
    assert_eq!(register_level(&mut index, 72, 106, 1147.0), 1);
    assert_eq!(register_level(&mut index, 72, 106, 2446.0), 2);
    assert_eq!(metastable_for_energy(&index, 72, 106, 0.0), 0);
    assert_eq!(metastable_for_energy(&index, 72, 106, 999.0), 0);
}

#[test]
fn level_index_from_bundles_matches_incremental_merge() {
    let bundles = HashMap::from([(
        "99tc".to_string(),
        bundle(
            vec![parent_row(&[]), parent_row(&[("p_energy", "0")])],
            vec![parent_row(&[("p_energy", "300.5")])],
        ),
    )]);
    let built = build_level_index_from_bundles(&bundles);
    let mut merged = LevelIndex::new();
    merge_levels_from_bundles(&bundles, &mut merged);
    assert_eq!(built, merged);
    assert_eq!(built[&(43, 56, 142_683)], 1);
    assert_eq!(built[&(43, 56, 300_500)], 2);
    assert!(!built.contains_key(&(43, 56, 0)));
}

#[test]
fn level_branches_merge_isomeric_and_beta_rows() {
    let gamma = parent_row(&[
        ("decay", "IT"),
        ("decay_%", "99.99"),
        ("d_z", "43"),
        ("d_n", "56"),
    ]);
    let beta = parent_row(&[
        ("decay", "B-"),
        ("decay_%", ""),
        ("intensity_beta", "0.0037"),
        ("d_z", "44"),
        ("d_n", "55"),
    ]);
    let branches = level_branches_from_bundle(
        142.683,
        &bundle(vec![gamma], vec![beta]),
        &LevelIndex::new(),
    );
    let by_mode: HashMap<DecayMode, f64> = branches
        .iter()
        .map(|branch| (branch.mode, branch.branching_pct))
        .collect();
    assert_eq!(by_mode[&DecayMode::Isomeric], 99.99);
    assert_eq!(by_mode[&DecayMode::BetaMinus], 0.0037);
}

#[test]
fn merge_decay_branches_keeps_strongest_duplicate() {
    let daughter = NuclideId::new(1, 1, 0);
    let make = |pct: f64| DecayBranch {
        mode: DecayMode::BetaMinus,
        branching_pct: pct,
        daughter,
    };
    let merged = merge_decay_branches(vec![make(10.0)], vec![make(40.0)]);
    assert_eq!(merged.len(), 1);
    assert_eq!(merged[0].branching_pct, 40.0);
}
