use crate::monitor::state::{MonitorSample, MonitorState};
use crate::smooth::{moving_average_f64, normalize_window};

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

pub fn plot_bounds(
    monitor: &MonitorState,
    series: PlotSeries,
    _smoothing_window: usize,
    window_secs: f64,
    points: &[[f64; 2]],
) -> PlotBounds {
    let latest = monitor
        .history
        .back()
        .map(|sample| sample.elapsed.as_secs_f64())
        .unwrap_or(0.0);
    let (x_min, x_max) = window_range(latest, window_secs);
    let visible: Vec<f64> = points.iter().map(|point| point[1]).collect();
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

pub fn window_range(latest_secs: f64, window_secs: f64) -> (f64, f64) {
    let window = window_secs.max(1.0);
    let x_max = latest_secs.max(window);
    (x_max - window, x_max)
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

fn apply_smoothing(points: Vec<[f64; 2]>, smoothing_window: usize) -> Vec<[f64; 2]> {
    let window = normalize_window(smoothing_window);
    if window <= 1 || points.len() < 2 {
        return points;
    }
    let xs: Vec<f64> = points.iter().map(|point| point[0]).collect();
    let ys: Vec<f64> = points.iter().map(|point| point[1]).collect();
    let smoothed = moving_average_f64(&ys, window);
    xs.into_iter().zip(smoothed).map(|(x, y)| [x, y]).collect()
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

#[cfg(test)]
#[path = "plot_bounds_tests.rs"]
mod plot_bounds_tests;
