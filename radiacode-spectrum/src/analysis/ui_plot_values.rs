use crate::analysis::compare::Comparison;
use crate::analysis::spectrum::{CollapsedSpectrum, counts_per_sec};
use crate::analysis::state::SampleAnalysis;
use crate::smooth::moving_average_f64;

pub fn peak_source_values(
    samples: &[SampleAnalysis],
    background: Option<&CollapsedSpectrum>,
    subtract_background: bool,
    smooth_window: usize,
) -> Option<(Vec<f64>, Vec<f64>)> {
    let axis = samples.first().map(|s| &s.spectrum).or(background)?;
    let energies = axis.energies_kev.clone();
    let values = if subtract_background {
        let background = background?;
        let sample = samples.first()?;
        let comparison = sample.comparison.as_ref()?;
        smoothed_net(sample, comparison, background, smooth_window)
    } else if let Some(sample) = samples.first() {
        smoothed_sample(sample, smooth_window)
    } else {
        smoothed_background(background?, smooth_window)
    };
    Some((energies, values))
}

pub fn smoothed_cps(counts: &[u64], live_time_secs: f64, smooth_window: usize) -> Vec<f64> {
    moving_average_f64(&counts_per_sec(counts, live_time_secs), smooth_window)
}

pub fn smoothed_sample(sample: &SampleAnalysis, smooth_window: usize) -> Vec<f64> {
    smoothed_cps(
        &sample.spectrum.counts,
        sample.spectrum.live_time_secs,
        smooth_window,
    )
}

pub fn smoothed_background(background: &CollapsedSpectrum, smooth_window: usize) -> Vec<f64> {
    smoothed_cps(&background.counts, background.live_time_secs, smooth_window)
}

pub fn smoothed_net(
    sample: &SampleAnalysis,
    _comparison: &Comparison,
    background: &CollapsedSpectrum,
    smooth_window: usize,
) -> Vec<f64> {
    let sample_values = smoothed_sample(sample, smooth_window);
    let background_values = smoothed_background(background, smooth_window);
    sample_values
        .iter()
        .zip(background_values.iter())
        .map(|(sample_value, background_value)| (sample_value - background_value).max(0.0))
        .collect()
}
