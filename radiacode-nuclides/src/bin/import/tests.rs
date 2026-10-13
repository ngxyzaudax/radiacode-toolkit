use std::collections::HashMap;

use radiacode_nuclides::{DecayBranch, DecayMode, NuclideId, RadiationKind};

use super::branches::{level_branches_from_bundle, merge_decay_branches};
use super::build::upsert_topology_candidate;
use super::bundle::{RadiationBundle, gammas_for_level};
use super::cli::flag_value;
use super::isomer::{daughter_isomer_candidates_from_bundle, isomer_candidates_from_bundle};
use super::level_index::{
    build_level_index_from_bundles, merge_levels_from_bundles, metastable_for_energy,
    register_level,
};
use super::parse::{display_name, parse_ground_states};
use super::parse_decay::{daughter_from_mode, parse_decay_mode, parse_radiation_row};
use super::{Candidate, CsvRow, LevelIndex};

fn row(pairs: &[(&str, &str)]) -> CsvRow {
    pairs
        .iter()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect()
}

fn parent_row(extra: &[(&str, &str)]) -> CsvRow {
    let mut base = row(&[
        ("p_z", "43"),
        ("p_n", "56"),
        ("p_symbol", "Tc"),
        ("p_energy", "142.683"),
        ("half_life_sec", "21624"),
        ("half_life", "6.01 h"),
    ]);
    base.extend(row(extra));
    base
}

fn bundle(gamma_rows: Vec<CsvRow>, beta_minus_rows: Vec<CsvRow>) -> RadiationBundle {
    RadiationBundle {
        gamma_rows,
        xray_rows: Vec::new(),
        beta_minus_rows,
        beta_plus_rows: Vec::new(),
    }
}

fn candidate(id: NuclideId, level_energy_kev: f64) -> Candidate {
    Candidate {
        id,
        symbol: "Tc".into(),
        mass_number: id.mass_number(),
        half_life_secs: None,
        half_life_text: "unknown".into(),
        decays: Vec::new(),
        force_include: false,
        level_energy_kev,
    }
}

#[test]
fn decay_mode_parsing_covers_known_labels() {
    assert_eq!(parse_decay_mode("IT"), DecayMode::Isomeric);
    assert_eq!(parse_decay_mode("SF"), DecayMode::SpontaneousFission);
    assert_eq!(parse_decay_mode("A"), DecayMode::Alpha);
    assert_eq!(parse_decay_mode("EC+B+"), DecayMode::ElectronCapture);
    assert_eq!(parse_decay_mode("B+"), DecayMode::BetaPlus);
    assert_eq!(parse_decay_mode("B-"), DecayMode::BetaMinus);
    assert_eq!(parse_decay_mode("2B-"), DecayMode::BetaMinus);
    assert_eq!(parse_decay_mode("p"), DecayMode::Proton);
    assert_eq!(parse_decay_mode("n"), DecayMode::Neutron);
    assert_eq!(parse_decay_mode(""), DecayMode::Unknown);
}

#[test]
fn daughter_from_mode_applies_nucleon_changes() {
    let parent = NuclideId::new(92, 146, 0);
    assert_eq!(
        daughter_from_mode(parent, DecayMode::Alpha),
        Some(NuclideId::new(90, 144, 0))
    );
    assert_eq!(
        daughter_from_mode(parent, DecayMode::BetaMinus),
        Some(NuclideId::new(93, 145, 0))
    );
    assert_eq!(
        daughter_from_mode(NuclideId::new(1, 0, 0), DecayMode::BetaPlus),
        None
    );
    assert_eq!(daughter_from_mode(parent, DecayMode::Unknown), None);
}

#[test]
fn display_name_marks_metastable_levels() {
    assert_eq!(display_name("Tc", 99, 0), "Tc-99");
    assert_eq!(display_name("Tc", 99, 1), "Tc-99m");
    assert_eq!(display_name("Hf", 178, 2), "Hf-178m2");
}

#[test]
fn radiation_row_filters_by_level_energy_and_intensity() {
    let line = row(&[
        ("p_energy", "0"),
        ("energy", "661.657"),
        ("intensity", "85.1"),
        ("decay", "B-"),
    ]);
    let parsed = parse_radiation_row(&line, RadiationKind::Gamma, 0.0).unwrap();
    assert_eq!(parsed.decay, DecayMode::BetaMinus);
    assert!(parse_radiation_row(&line, RadiationKind::Gamma, 142.0).is_none());
    let faint = row(&[("p_energy", "0"), ("energy", "10"), ("intensity", "0.001")]);
    assert!(parse_radiation_row(&faint, RadiationKind::XRay, 0.0).is_none());
}

#[test]
fn ground_state_parsing_skips_excited_rows_and_empty_symbols() {
    let rows = vec![
        row(&[("z", "55"), ("n", "82"), ("symbol", "Cs"), ("energy", "0")]),
        row(&[
            ("z", "55"),
            ("n", "82"),
            ("symbol", "Cs"),
            ("energy", "12.5"),
        ]),
        row(&[("z", "55"), ("n", "82"), ("symbol", " "), ("energy", "0")]),
        row(&[("z", "0"), ("n", "1"), ("symbol", "n"), ("energy", "0")]),
    ];
    let candidates = parse_ground_states(&rows);
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0].id, NuclideId::new(55, 82, 0));
}

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

#[test]
fn flag_value_parses_prefixed_arguments() {
    let args = vec!["--workers=12".to_string(), "--limit=abc".to_string()];
    assert_eq!(flag_value(&args, "--workers="), Some(12));
    assert_eq!(flag_value(&args, "--limit="), None);
}
