use crate::catalog::{SfrValueKind, parse_sfr_file, validate_catalog};
use crate::command::VirtSfr;

use crate::types::channel_to_energy;
use crate::units::{CountDisplayUnit, DoseDisplayUnit};

#[test]
fn channel_to_energy_is_quadratic() {
    assert_eq!(channel_to_energy(0, 1.0, 2.0, 3.0), 1.0);
    assert_eq!(channel_to_energy(2, 1.0, 2.0, 3.0), 17.0);
}

#[test]
fn validate_catalog_reports_missing_and_mismatched_registers() {
    assert!(validate_catalog("no entries here").is_none());
    let sfr = "[VSFR_DEVICE_CTRL]\nAddr=0x00000500\nSize=4\nType=4\n\
               [VSFR_DISP_BRT]\nAddr=0x00000511\nSize=4\nType=5\n";
    let drifts = validate_catalog(sfr).expect("drifts");
    assert!(
        drifts
            .iter()
            .any(|d| d.register == VirtSfr::DispBrt && d.message.contains("device type"))
    );
    assert!(
        drifts
            .iter()
            .any(|d| d.register == VirtSfr::DeviceTime && d.message.contains("missing"))
    );
    assert!(!drifts.iter().any(|d| d.register == VirtSfr::DeviceCtrl));
}

#[test]
fn validate_catalog_accepts_unknown_device_types() {
    let sfr = "[VSFR_DEVICE_CTRL]\nAddr=0x00000500\nSize=4\nType=7\n";
    let drifts = validate_catalog(sfr).expect("drifts");
    assert!(!drifts.iter().any(|d| d.register == VirtSfr::DeviceCtrl));
}

#[test]
fn sfr_file_parser_handles_all_fields_and_bad_values() {
    let entries = parse_sfr_file(
        "noise\n[VSFR_A]\nAddr=0X00000511\nSize=x\nType=2\nSigned=1\n[OTHER]\nAddr=0x1\n[VSFR_B]\nAddr=zz\n",
    );
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].address, 0x511);
    assert_eq!(entries[0].size, 0);
    assert_eq!(entries[0].value_kind, SfrValueKind::U16);
    assert!(entries[0].signed);
    assert_eq!(entries[1].address, 0);
    assert_eq!(entries[1].value_kind, SfrValueKind::Unknown);
}

#[test]
fn display_unit_flags_round_trip() {
    for unit in [
        DoseDisplayUnit::MicroSievertPerHour,
        DoseDisplayUnit::MicroRoentgenPerHour,
    ] {
        assert_eq!(
            DoseDisplayUnit::from_device_flag(unit.to_device_flag()),
            unit
        );
    }
    assert!(DoseDisplayUnit::MicroSievertPerHour.is_sv());
    assert!(!DoseDisplayUnit::MicroRoentgenPerHour.is_sv());
    for unit in [CountDisplayUnit::Cpm, CountDisplayUnit::Cps] {
        assert_eq!(
            CountDisplayUnit::from_device_flag(unit.to_device_flag()),
            unit
        );
    }
}
