use radiacode_nuclides::{DecayMode, NuclideId, RadiationKind};

use super::cli::flag_value;
use super::parse::{display_name, parse_ground_states};
use super::parse_decay::{daughter_from_mode, parse_decay_mode, parse_radiation_row};
use super::test_fixtures::row;

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
fn flag_value_parses_prefixed_arguments() {
    let args = vec!["--workers=12".to_string(), "--limit=abc".to_string()];
    assert_eq!(flag_value(&args, "--workers="), Some(12));
    assert_eq!(flag_value(&args, "--limit="), None);
}
