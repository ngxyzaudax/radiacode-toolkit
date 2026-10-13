use radiacode_nuclides::RadiationKind;

use super::bundle::{RadiationBundle, gammas_for_level};
use super::test_fixtures::parent_row;

fn radiation_row(energy: &str, intensity: &str) -> super::CsvRow {
    parent_row(&[("energy", energy), ("intensity", intensity)])
}

fn bundle_with_xray_twin() -> RadiationBundle {
    RadiationBundle {
        gamma_rows: vec![
            radiation_row("140.511", "89"),
            radiation_row("18.367", "4.03"),
        ],
        xray_rows: vec![radiation_row("18.367", "4.03")],
        beta_minus_rows: Vec::new(),
        beta_plus_rows: Vec::new(),
    }
}

#[test]
fn gamma_rows_duplicating_xray_rows_are_dropped() {
    let lines = gammas_for_level(&bundle_with_xray_twin(), 142.683);
    assert_eq!(lines.len(), 2);
    let xray = lines.iter().filter(|line| line.energy_kev == 18.367);
    assert!(xray.clone().count() == 1);
    assert!(
        xray.into_iter()
            .all(|line| line.kind == RadiationKind::XRay)
    );
}

#[test]
fn distinct_gamma_rows_are_kept_as_gamma() {
    let lines = gammas_for_level(&bundle_with_xray_twin(), 142.683);
    assert!(
        lines
            .iter()
            .any(|line| line.energy_kev == 140.511 && line.kind == RadiationKind::Gamma)
    );
}
