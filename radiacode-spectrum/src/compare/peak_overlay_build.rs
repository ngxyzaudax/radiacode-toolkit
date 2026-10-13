use crate::app_config::AppConfig;
use crate::compare::spectrum::CollapsedSpectrum;
use crate::compare::state::ComparedSample;
use crate::compare::ui_plot_bars::PlotPeakOverlay;
use crate::compare::ui_plot_values::{smoothed_background, smoothed_net, smoothed_sample};
use crate::identify::{analysis_window_from_energies, analyze_peaks};
use crate::peaks::DetectionParams;
use crate::peaks::{PeakMemoKey, detect_peaks, peaks_from_collapsed};

pub struct BuiltPeakOverlay {
    pub peak_count: usize,
    pub identifications: Vec<radiacode_nuclides::PeakIdentification>,
    pub sources: radiacode_nuclides::SourceSummary,
    pub display_values: Vec<f64>,
    pub energies: Vec<f64>,
}

impl BuiltPeakOverlay {
    pub fn plot_overlay<'a>(
        &'a self,
        y_scale: crate::scale::YScale,
        log_floor: f64,
    ) -> PlotPeakOverlay<'a> {
        PlotPeakOverlay {
            identifications: &self.identifications,
            display_values: &self.display_values,
            energies: &self.energies,
            y_scale,
            log_floor,
        }
    }
}

pub fn build_peak_overlay(
    samples: &[ComparedSample],
    background: Option<&CollapsedSpectrum>,
    subtract_background: bool,
    show_peaks: bool,
    smooth_window: usize,
    config: &AppConfig,
    peak_memo: &mut crate::peaks::PeakMemo,
) -> Option<BuiltPeakOverlay> {
    if !show_peaks {
        return None;
    }
    let params = DetectionParams::from_app_config(config);
    let data_token = peak_overlay_token(samples, background, subtract_background);
    let key = PeakMemoKey::new(data_token, params);
    let peaks = peak_memo
        .get_or_compute(key, || {
            detect_overlay_peaks(samples, background, subtract_background, params)
        })
        .to_vec();
    let (energies, display_values) =
        overlay_display_data(samples, background, subtract_background, smooth_window)?;
    let analysis = analyze_peaks(&peaks, config, analysis_window_from_energies(&energies));
    Some(BuiltPeakOverlay {
        peak_count: analysis.peaks.len(),
        identifications: analysis.identifications,
        sources: analysis.sources,
        display_values,
        energies,
    })
}

fn overlay_display_data(
    samples: &[ComparedSample],
    background: Option<&CollapsedSpectrum>,
    subtract_background: bool,
    smooth_window: usize,
) -> Option<(Vec<f64>, Vec<f64>)> {
    match (subtract_background, samples.first(), background) {
        (true, Some(sample), Some(background)) if sample.subtraction.is_some() => Some((
            sample.spectrum.energies_kev.clone(),
            smoothed_net(sample, background, smooth_window),
        )),
        (true, _, _) => None,
        (false, Some(sample), _) => Some((
            sample.spectrum.energies_kev.clone(),
            smoothed_sample(sample, smooth_window),
        )),
        (false, None, Some(background)) => Some((
            background.energies_kev.clone(),
            smoothed_background(background, smooth_window),
        )),
        (false, None, None) => None,
    }
}

fn peak_overlay_token(
    samples: &[ComparedSample],
    background: Option<&CollapsedSpectrum>,
    subtract_background: bool,
) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    subtract_background.hash(&mut hasher);
    if let Some(sample) = samples.first() {
        sample.spectrum.counts.len().hash(&mut hasher);
        sample
            .spectrum
            .counts
            .iter()
            .take(8)
            .for_each(|value| value.hash(&mut hasher));
    }
    if let Some(background) = background {
        background.counts.len().hash(&mut hasher);
        background
            .counts
            .iter()
            .take(8)
            .for_each(|value| value.hash(&mut hasher));
    }
    hasher.finish()
}

fn detect_overlay_peaks(
    samples: &[ComparedSample],
    background: Option<&CollapsedSpectrum>,
    subtract_background: bool,
    params: DetectionParams,
) -> Vec<crate::peaks::DetectedPeak> {
    if subtract_background {
        let Some(sample) = samples.first() else {
            return Vec::new();
        };
        let Some(subtraction) = sample.subtraction.as_ref() else {
            return Vec::new();
        };
        let energies = &sample.spectrum.energies_kev;
        let counts: Vec<f64> = subtraction
            .net_counts
            .iter()
            .map(|value| value.max(0.0))
            .collect();
        return detect_peaks(energies, &counts, params);
    }
    if let Some(sample) = samples.first() {
        return peaks_from_collapsed(&sample.spectrum, params);
    }
    background
        .map(|background| peaks_from_collapsed(background, params))
        .unwrap_or_default()
}
