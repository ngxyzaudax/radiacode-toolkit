use radiacode_core::{AccumulatedDose, AlarmLimits, CountDisplayUnit, DoseDisplayUnit};

use crate::dosimeter::plot_bounds::{dose_points, plot_bounds};
use crate::dosimeter::state::DosimeterState;
use crate::monitor::AlarmLevel;
use crate::persist::json_store::use_test_data_dir;

fn isolated() -> tempfile::TempDir {
    let temp = tempfile::tempdir().expect("tempdir");
    use_test_data_dir(temp.path().to_path_buf());
    temp
}

fn sample(duration_secs: u32, dose: f32) -> AccumulatedDose {
    AccumulatedDose {
        dose,
        duration_secs,
        dose_unit: DoseDisplayUnit::MicroSievertPerHour,
    }
}

fn limits(l1_dose: f32, l2_dose: f32) -> AlarmLimits {
    AlarmLimits {
        l1_count_rate: 0.0,
        l2_count_rate: 0.0,
        l1_dose_rate: 0.0,
        l2_dose_rate: 0.0,
        l1_dose,
        l2_dose,
        dose_unit: DoseDisplayUnit::MicroSievertPerHour,
        count_unit: CountDisplayUnit::Cps,
    }
}

#[test]
fn samples_are_thinned_by_minimum_spacing_and_negative_dose_clamped() {
    let _dir = isolated();
    let mut state = DosimeterState::new();
    state.on_connect("SN1");
    state.push_sample(sample(0, -1.0));
    state.push_sample(sample(5, 0.5));
    state.push_sample(sample(10, 1.0));
    assert_eq!(state.history.len(), 2);
    assert_eq!(state.history[0].dose, 0.0);
    assert_eq!(state.latest.map(|dose| dose.duration_secs), Some(10));
    assert_eq!(state.status, "Live dosimeter");
}

#[test]
fn session_restart_clears_history() {
    let _dir = isolated();
    let mut state = DosimeterState::new();
    state.on_connect("SN1");
    state.push_sample(sample(100, 1.0));
    state.push_sample(sample(200, 2.0));
    state.push_sample(sample(20, 0.1));
    assert_eq!(state.history.len(), 1);
    assert_eq!(state.history[0].duration_secs, 20);
}

#[test]
fn history_persists_across_reconnect_for_same_serial() {
    let _dir = isolated();
    let mut state = DosimeterState::new();
    state.on_connect("SN1");
    state.push_sample(sample(100, 1.0));
    state.push_sample(sample(200, 2.0));
    state.on_disconnect();
    assert!(state.history.is_empty());
    assert!(state.latest.is_none());
    state.on_connect("SN1");
    assert_eq!(state.history.len(), 2);
    assert_eq!(state.latest.map(|dose| dose.dose), Some(2.0));
    assert!(state.status.starts_with("Restored 2"));
    state.on_connect("OTHER");
    assert!(state.history.is_empty());
}

#[test]
fn reset_clears_history_and_stored_file() {
    let _dir = isolated();
    let mut state = DosimeterState::new();
    state.on_connect("SN1");
    state.push_sample(sample(100, 1.0));
    state.on_reset();
    assert!(state.history.is_empty());
    state.on_disconnect();
    state.on_connect("SN1");
    assert!(state.history.is_empty());
}

#[test]
fn dose_alarm_level_follows_limits() {
    let _dir = isolated();
    let mut state = DosimeterState::new();
    assert_eq!(state.dose_alarm_level(), AlarmLevel::Normal);
    state.apply_limits(limits(1.0, 2.0));
    state.push_sample(sample(10, 1.5));
    assert_eq!(state.dose_alarm_level(), AlarmLevel::Warning);
    state.push_sample(sample(30, 2.5));
    assert_eq!(state.dose_alarm_level(), AlarmLevel::Danger);
}

#[test]
fn plot_bounds_cover_history_and_alarm_limits_with_headroom() {
    let _dir = isolated();
    let mut state = DosimeterState::new();
    let empty = plot_bounds(&state);
    assert_eq!(empty.x_max, 60.0);
    assert!((empty.y_max - 0.0012).abs() < 1e-9);
    state.apply_limits(limits(1.0, 4.0));
    state.push_sample(sample(120, 2.0));
    let bounds = plot_bounds(&state);
    assert_eq!(bounds.x_max, 120.0);
    assert!((bounds.y_max - 4.8).abs() < 1e-9);
    let points = dose_points(&state, bounds);
    assert_eq!(points, vec![[120.0, 2.0]]);
}
