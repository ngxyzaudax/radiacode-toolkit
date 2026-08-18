use radiacode_core::{CountDisplayUnit, DeviceTicks, DoseDisplayUnit, TimedRates};

use super::MonitorState;
use std::time::Duration;

fn timed(ticks: i32, dose: f32, count: f32) -> TimedRates {
    TimedRates {
        device_ts: DeviceTicks::new(ticks),
        dose_rate: dose,
        count_rate: count,
        dose_rate_err_pct: 0.0,
        count_rate_err_pct: 0.0,
        dose_unit: DoseDisplayUnit::MicroSievertPerHour,
        count_unit: CountDisplayUnit::Cps,
    }
}

#[test]
fn negative_device_ticks_advance_elapsed() {
    let mut monitor = MonitorState::default_for_tests();
    monitor.set_wall_elapsed_for_tests(Duration::from_secs(2));
    monitor.push_poll(&[timed(-3387, 0.08, 15.0)], 0, 0, 0, &[]);
    monitor.set_wall_elapsed_for_tests(Duration::from_secs(3));
    monitor.push_poll(&[timed(-3287, 0.09, 16.0)], 0, 0, 0, &[]);
    assert_eq!(monitor.history.len(), 2);
    assert_eq!(monitor.history[0].elapsed.as_secs(), 0);
    assert_eq!(monitor.history[1].elapsed.as_secs(), 1);
}

#[test]
fn older_device_ticks_do_not_rewind_epoch() {
    let mut monitor = MonitorState::default_for_tests();
    monitor.set_wall_elapsed_for_tests(Duration::from_secs(10));
    monitor.push_poll(&[timed(1000, 0.08, 15.0)], 0, 0, 0, &[]);
    monitor.set_wall_elapsed_for_tests(Duration::from_secs(20));
    monitor.push_poll(&[timed(2000, 0.09, 16.0)], 0, 0, 0, &[]);
    monitor.set_wall_elapsed_for_tests(Duration::from_secs(30));
    monitor.push_poll(&[timed(500, 0.5, 80.0)], 0, 0, 0, &[]);
    assert_eq!(monitor.history.len(), 2);
    assert_eq!(monitor.history[1].elapsed.as_secs(), 10);
    assert!((monitor.latest.unwrap().count_rate - 80.0).abs() < 0.01);
}

#[test]
fn backlog_jump_is_clamped_to_wall_time() {
    let mut monitor = MonitorState::default_for_tests();
    monitor.set_wall_elapsed_for_tests(Duration::ZERO);
    monitor.push_poll(&[timed(0, 0.08, 15.0)], 0, 0, 0, &[]);
    monitor.set_wall_elapsed_for_tests(Duration::from_secs(100));
    monitor.push_poll(&[timed(10_000, 0.08, 15.0)], 0, 0, 0, &[]);
    monitor.set_wall_elapsed_for_tests(Duration::from_secs(105));
    monitor.push_poll(
        &[
            timed(11_000, 0.08, 15.0),
            timed(15_000, 0.08, 15.0),
            timed(18_000, 1.5, 200.0),
        ],
        0,
        0,
        0,
        &[],
    );
    assert_eq!(monitor.history.len(), 3);
    let latest = monitor.history.back().unwrap();
    assert!(latest.elapsed.as_secs_f64() <= 105.75);
    assert!(latest.elapsed.as_secs_f64() >= 105.0);
    assert!((latest.count_rate - 200.0).abs() < 0.01);
    assert!(latest.elapsed.as_secs_f64() - 100.0 < 6.0);
}

#[test]
fn poll_keeps_only_newest_rate_from_batch() {
    let mut monitor = MonitorState::default_for_tests();
    monitor.set_wall_elapsed_for_tests(Duration::from_secs(3));
    monitor.push_poll(
        &[
            timed(0, 0.08, 15.0),
            timed(100, 0.09, 16.0),
            timed(200, 0.5, 90.0),
        ],
        0,
        0,
        0,
        &[],
    );
    assert_eq!(monitor.history.len(), 1);
    assert!((monitor.history[0].count_rate - 90.0).abs() < 0.01);
}
