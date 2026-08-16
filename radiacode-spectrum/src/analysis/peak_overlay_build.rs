use crate::analysis::spectrum::CollapsedSpectrum;
use crate::analysis::state::SampleAnalysis;
use crate::analysis::ui_plot_bars::PlotPeakOverlay;
use crate::app_config::AppConfig;
use crate::identify::analyze_peaks;
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
    samples: &[SampleAnalysis],
    background: Option<&CollapsedSpectrum>,
    subtract_background: bool,
    show_peaks: bool,
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
    let analysis = analyze_peaks(&peaks, config);
    let (energies, display_values) = if subtract_background {
        let sample = samples.first()?;
        let comparison = sample.comparison.as_ref()?;
        let energies = sample.spectrum.energies_kev.clone();
        let display_values: Vec<f64> = comparison
            .net_counts
            .iter()
            .map(|value| value.max(0.0))
            .collect();
        (energies, display_values)
    } else if let Some(sample) = samples.first() {
        let display_values = sample
            .spectrum
            .counts
            .iter()
            .map(|&value| value as f64)
            .collect();
        (sample.spectrum.energies_kev.clone(), display_values)
    } else {
        let background = background?;
        let display_values = background
            .counts
            .iter()
            .map(|&value| value as f64)
            .collect();
        (background.energies_kev.clone(), display_values)
    };
    Some(BuiltPeakOverlay {
        peak_count: analysis.peaks.len(),
        identifications: analysis.identifications,
        sources: analysis.sources,
        display_values,
        energies,
    })
}

fn peak_overlay_token(
    samples: &[SampleAnalysis],
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
    samples: &[SampleAnalysis],
    background: Option<&CollapsedSpectrum>,
    subtract_background: bool,
    params: DetectionParams,
) -> Vec<crate::peaks::DetectedPeak> {
    if subtract_background {
        let Some(sample) = samples.first() else {
            return Vec::new();
        };
        let Some(comparison) = sample.comparison.as_ref() else {
            return Vec::new();
        };
        let energies = &sample.spectrum.energies_kev;
        let counts: Vec<f64> = comparison
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
