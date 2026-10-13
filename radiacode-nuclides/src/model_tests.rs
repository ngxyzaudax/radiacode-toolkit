use crate::chain::decay_mode_label;
use crate::elements::{element_symbol, nuclide_display_name};
use crate::model::{DecayMode, GammaLine, NuclideId, RadiationKind};

const ALL_MODES: [(DecayMode, &str); 9] = [
    (DecayMode::Alpha, "alpha"),
    (DecayMode::BetaMinus, "beta-"),
    (DecayMode::BetaPlus, "beta+"),
    (DecayMode::ElectronCapture, "EC"),
    (DecayMode::Isomeric, "IT"),
    (DecayMode::SpontaneousFission, "SF"),
    (DecayMode::Proton, "p"),
    (DecayMode::Neutron, "n"),
    (DecayMode::Unknown, "?"),
];

#[test]
fn mass_number_sums_protons_and_neutrons() {
    assert_eq!(NuclideId::new(92, 146, 0).mass_number(), 238);
    assert_eq!(NuclideId::new(1, 0, 0).mass_number(), 1);
}

#[test]
fn decay_mode_labels_are_distinct_and_stable() {
    for (mode, label) in ALL_MODES {
        assert_eq!(decay_mode_label(mode), label);
    }
}

#[test]
fn element_symbols_cover_periodic_table_bounds() {
    assert_eq!(element_symbol(0), "?");
    assert_eq!(element_symbol(1), "H");
    assert_eq!(element_symbol(92), "U");
    assert_eq!(element_symbol(118), "Og");
    assert_eq!(element_symbol(119), "?");
    assert_eq!(element_symbol(255), "?");
}

#[test]
fn display_name_joins_symbol_and_mass_number() {
    assert_eq!(nuclide_display_name(55, 137), "Cs-137");
    assert_eq!(nuclide_display_name(200, 1), "?-1");
}

#[test]
fn radiation_kind_labels() {
    assert_eq!(RadiationKind::Gamma.label(), "γ");
    assert_eq!(RadiationKind::XRay.label(), "X");
}

#[test]
fn gamma_line_defaults_to_gamma_kind_when_missing() {
    let json = r#"{"energy_kev":661.7,"intensity_pct":85.1,"decay":"beta_minus"}"#;
    let line: GammaLine = serde_json::from_str(json).expect("gamma line");
    assert_eq!(line.kind, RadiationKind::Gamma);
    assert_eq!(line.decay, DecayMode::BetaMinus);
}

#[test]
fn gamma_line_round_trips_explicit_kind() {
    let line = GammaLine {
        energy_kev: 59.5,
        intensity_pct: 35.9,
        decay: DecayMode::Alpha,
        kind: RadiationKind::XRay,
    };
    let json = serde_json::to_string(&line).expect("serialize");
    assert!(json.contains("\"kind\":\"x_ray\""));
    let decoded: GammaLine = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(decoded, line);
}
