use crate::monitor::state::{MonitorSample, MonitorState};
use crate::smooth::{moving_average_f64, normalize_window};

const WINDOW_SECS: f64 = 120.0;
const Y_HEADROOM: f64 = 0.2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlotBounds {
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
}

#[derive(Clone, Copy)]
pub enum PlotSeries {
    Dose,
    Count,
}

pub fn series_points(
    monitor: &MonitorState,
    series: PlotSeries,
    bounds: PlotBounds,
    smoothing_window: usize,
) -> Vec<[f64; 2]> {
    let visible: Vec<[f64; 2]> = monitor
        .history
        .iter()
        .filter(|sample| sample_in_window(sample, bounds))
        .map(|sample| [elapsed_secs(sample), series_value(sample, series)])
        .collect();
    apply_smoothing(visible, smoothing_window)
}

pub fn plot_bounds(monitor: &MonitorState, series: PlotSeries, smoothing_window: usize) -> PlotBounds {
    let x_max = monitor
        .history
        .back()
        .map(elapsed_secs)
        .unwrap_or(0.0)
        .max(1.0);
    let oldest = monitor
        .history
        .front()
        .map(elapsed_secs)
        .unwrap_or(0.0);
    let x_min = window_x_min(oldest, x_max);
    let bounds = PlotBounds {
        x_min,
        x_max,
        y_min: 0.0,
        y_max: 1.0,
    };
    let visible: Vec<f64> = series_points(monitor, series, bounds, smoothing_window)
        .into_iter()
        .map(|point| point[1])
        .collect();
    let alarm_max = monitor
        .limits
        .map(|limits| alarm_ceiling(limits, series))
        .unwrap_or(0.0);
    PlotBounds {
        x_min,
        x_max,
        y_min: 0.0,
        y_max: upper_y_bound(&visible, alarm_max, series),
    }
}

fn apply_smoothing(points: Vec<[f64; 2]>, smoothing_window: usize) -> Vec<[f64; 2]> {
    let window = normalize_window(smoothing_window);
    if window <= 1 || points.len() < 2 {
        return points;
    }
    let xs: Vec<f64> = points.iter().map(|point| point[0]).collect();
    let ys: Vec<f64> = points.iter().map(|point| point[1]).collect();
    let smoothed = moving_average_f64(&ys, window);
    xs.into_iter()
        .zip(smoothed)
        .map(|(x, y)| [x, y])
        .collect()
}

fn window_x_min(oldest: f64, x_max: f64) -> f64 {
    let scrolled = (x_max - WINDOW_SECS).max(0.0);
    if oldest > scrolled {
        oldest
    } else {
        scrolled
    }
}

fn alarm_ceiling(limits: radiacode_core::AlarmLimits, series: PlotSeries) -> f64 {
    match series {
        PlotSeries::Dose => f64::from(limits.l1_dose_rate.max(limits.l2_dose_rate).max(0.0)),
        PlotSeries::Count => f64::from(limits.l1_count_rate.max(limits.l2_count_rate).max(0.0)),
    }
}

fn upper_y_bound(values: &[f64], alarm_max: f64, series: PlotSeries) -> f64 {
    let data_max = values.iter().copied().fold(0.0_f64, f64::max);
    let floor = match series {
        PlotSeries::Dose => 0.1,
        PlotSeries::Count => 1.0,
    };
    let peak = data_max.max(alarm_max);
    (peak * (1.0 + Y_HEADROOM)).max(floor)
}

fn sample_in_window(sample: &MonitorSample, bounds: PlotBounds) -> bool {
    let seconds = elapsed_secs(sample);
    seconds >= bounds.x_min && seconds <= bounds.x_max
}

fn elapsed_secs(sample: &MonitorSample) -> f64 {
    sample.elapsed.as_secs_f64()
}

fn series_value(sample: &MonitorSample, series: PlotSeries) -> f64 {
    match series {
        PlotSeries::Dose => f64::from(sample.dose_rate.max(0.0)),
        PlotSeries::Count => f64::from(sample.count_rate.max(0.0)),
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use radiacode_core::{CountDisplayUnit, DoseDisplayUnit, DeviceTicks};

    use super::{plot_bounds, PlotSeries};
    use crate::monitor::state::{MonitorSample, MonitorState};

    fn sample(seconds: f64, dose: f32, count: f32) -> MonitorSample {
        MonitorSample {
            dose_rate: dose,
            count_rate: count,
            dose_rate_err_pct: 0.0,
            count_rate_err_pct: 0.0,
            elapsed: Duration::from_secs_f64(seconds),
        }
    }

    fn timed(seconds: f64, dose: f32, count: f32) -> radiacode_core::TimedRates {
        radiacode_core::TimedRates {
            device_ts: DeviceTicks::new((seconds * 100.0) as i32),
            dose_rate: dose,
            count_rate: count,
            dose_rate_err_pct: 0.0,
            count_rate_err_pct: 0.0,
            dose_unit: DoseDisplayUnit::MicroSievertPerHour,
            count_unit: CountDisplayUnit::Cps,
        }
    }

    #[test]
    fn window_scrolls_with_latest_sample() {
        let mut monitor = MonitorState::default_for_tests();
        monitor.push_poll(&[timed(10.0, 1.0, 10.0)], 0, 0, &[]);
        monitor.push_poll(&[timed(150.0, 2.0, 20.0)], 0, 0, &[]);
        let bounds = plot_bounds(&monitor, PlotSeries::Dose, 1);
        assert!((bounds.x_min - 20.0).abs() < 0.01);
        assert!((bounds.x_max - 140.0).abs() < 0.01);
    }

    #[test]
    fn y_axis_always_starts_at_zero() {
        let mut monitor = MonitorState::default_for_tests();
        monitor.history.push_back(sample(1.0, 10.0, 100.0));
        monitor.history.push_back(sample(2.0, 12.0, 120.0));
        let bounds = plot_bounds(&monitor, PlotSeries::Dose, 1);
        assert_eq!(bounds.y_min, 0.0);
        assert!(bounds.y_max > 12.0);
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
            dose_unit: DoseDisplayUnit::MicroSievertPerHour,
            count_unit: CountDisplayUnit::Cps,
        });
        monitor.history.push_back(sample(1.0, 0.09, 17.0));
        monitor.history.push_back(sample(2.0, 0.09, 17.0));
        let bounds = plot_bounds(&monitor, PlotSeries::Dose, 1);
        assert_eq!(bounds.y_min, 0.0);
        assert!((bounds.y_max - 0.36).abs() < 0.001);
    }
}
