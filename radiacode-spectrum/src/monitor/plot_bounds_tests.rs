use crate::monitor::plot_bounds::{
    PlotBounds, PlotSeries, plot_bounds, series_points, window_range,
};
use crate::monitor::state::{MonitorSample, MonitorState};

const WINDOW_SECS: f64 = 120.0;

fn sample(seconds: f64, dose: f32, count: f32) -> MonitorSample {
    MonitorSample {
        dose_rate: dose,
        count_rate: count,
        dose_rate_err_pct: 0.0,
        count_rate_err_pct: 0.0,
        elapsed: std::time::Duration::from_secs_f64(seconds),
    }
}

fn timed(seconds: f64, dose: f32, count: f32) -> radiacode_core::TimedRates {
    radiacode_core::TimedRates {
        device_ts: radiacode_core::DeviceTicks::new((seconds * 100.0) as i32),
        dose_rate: dose,
        count_rate: count,
        dose_rate_err_pct: 0.0,
        count_rate_err_pct: 0.0,
        dose_unit: radiacode_core::DoseDisplayUnit::MicroSievertPerHour,
        count_unit: radiacode_core::CountDisplayUnit::Cps,
    }
}

fn bounds_for(
    monitor: &MonitorState,
    series: PlotSeries,
) -> crate::monitor::plot_bounds::PlotBounds {
    let latest = monitor
        .history
        .back()
        .map(|sample| sample.elapsed.as_secs_f64())
        .unwrap_or(0.0);
    let (x_min, x_max) = window_range(latest, WINDOW_SECS);
    let draft = PlotBounds {
        x_min,
        x_max,
        y_min: 0.0,
        y_max: 1.0,
    };
    let points = series_points(monitor, series, draft, 1);
    plot_bounds(monitor, series, 1, WINDOW_SECS, &points)
}

#[test]
fn window_is_full_width_with_one_sample() {
    let mut monitor = MonitorState::default_for_tests();
    monitor.history.push_back(sample(1.0, 1.0, 10.0));
    let bounds = bounds_for(&monitor, PlotSeries::Dose);
    assert!((bounds.x_min - 0.0).abs() < 0.01);
    assert!((bounds.x_max - WINDOW_SECS).abs() < 0.01);
}

#[test]
fn window_scrolls_with_latest_sample() {
    let mut monitor = MonitorState::default_for_tests();
    monitor.set_wall_elapsed_for_tests(std::time::Duration::from_secs(10));
    monitor.push_poll(&[timed(10.0, 1.0, 10.0)], 0, 0, 0, &[]);
    monitor.set_wall_elapsed_for_tests(std::time::Duration::from_secs(150));
    monitor.push_poll(&[timed(150.0, 2.0, 20.0)], 0, 0, 0, &[]);
    let bounds = bounds_for(&monitor, PlotSeries::Dose);
    assert!((bounds.x_min - 20.0).abs() < 0.01);
    assert!((bounds.x_max - 140.0).abs() < 0.01);
}

#[test]
fn y_max_uses_alarm_when_data_is_lower() {
    let mut monitor = MonitorState::default_for_tests();
    monitor.limits = Some(radiacode_core::AlarmLimits {
        l1_count_rate: 20.0,
        l2_count_rate: 40.0,
        l1_dose_rate: 0.15,
        l2_dose_rate: 0.3,
        l1_dose: 100.0,
        l2_dose: 200.0,
        dose_unit: radiacode_core::DoseDisplayUnit::MicroSievertPerHour,
        count_unit: radiacode_core::CountDisplayUnit::Cps,
    });
    monitor.history.push_back(sample(1.0, 0.09, 17.0));
    monitor.history.push_back(sample(2.0, 0.09, 17.0));
    let bounds = bounds_for(&monitor, PlotSeries::Dose);
    assert_eq!(bounds.y_min, 0.0);
    assert!((bounds.y_max - 0.36).abs() < 0.001);
}
