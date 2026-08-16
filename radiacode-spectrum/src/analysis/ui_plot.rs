use egui::RichText;
use egui::Ui;

use crate::analysis::spectrum::CollapsedSpectrum;
use crate::analysis::state::SampleAnalysis;
use crate::analysis::ui_plot_bars::{PlotPeakOverlay, owned_series, shared_log_floor, show_owned_series};
use crate::analysis::ui_plot_legend::draw_legend;
use crate::analysis::ui_plot_values::{smoothed_background, smoothed_net, smoothed_sample};
use crate::app_config::AppConfig;
use crate::identify::{analyze_peaks, detection_params_from_config};
use crate::peak_overlay::{SpectrumPlotAction, draw_source_chips};
use crate::peaks::{detect_peaks, peaks_from_collapsed};
use crate::scale::{HistogramStyle, YScale};
use crate::theme::{ANALYSIS_BACKGROUND, MUTED, analysis_sample_color};

pub struct AnalysisPlotProps<'a> {
    pub samples: &'a [SampleAnalysis],
    pub background: Option<&'a CollapsedSpectrum>,
    pub y_scale: YScale,
    pub smooth_window: usize,
    pub style: HistogramStyle,
    pub subtract_background: bool,
    pub show_peaks: bool,
    pub config: &'a AppConfig,
}

pub fn draw_analysis_plots(
    ui: &mut Ui,
    props: AnalysisPlotProps<'_>,
) -> Option<SpectrumPlotAction> {
    if props.samples.is_empty() && props.background.is_none() {
        return None;
    }
    let Some(axis) = props
        .samples
        .first()
        .map(|sample| &sample.spectrum)
        .or(props.background)
    else {
        return None;
    };
    let energies = &axis.energies_kev;
    let width = axis.a1 as f64;
    let show_net = props.subtract_background && props.background.is_some();
    draw_legend(ui, props.samples, props.background.is_some() && !show_net);
    ui.label(if show_net {
        "Net (sample − background)"
    } else {
        "Overlay"
    });
    let peak_overlay = build_peak_overlay(
        props.samples,
        props.background,
        props.subtract_background,
        props.show_peaks,
        props.config,
    );
    if show_net {
        draw_net_plot(
            ui,
            energies,
            width,
            props.samples,
            props.background,
            props.y_scale,
            props.smooth_window,
            props.style,
            peak_overlay.as_ref(),
        );
    } else {
        draw_overlay_plot(
            ui,
            energies,
            width,
            props.samples,
            props.background,
            props.y_scale,
            props.smooth_window,
            props.style,
            peak_overlay.as_ref(),
        );
    }
    peak_overlay.and_then(|overlay| draw_source_chips(ui, &overlay.sources))
}

struct BuiltPeakOverlay {
    identifications: Vec<radiacode_nuclides::PeakIdentification>,
    sources: radiacode_nuclides::SourceSummary,
    display_values: Vec<f64>,
    energies: Vec<f64>,
}

impl BuiltPeakOverlay {
    fn plot_overlay<'a>(&'a self, y_scale: YScale, log_floor: f64) -> PlotPeakOverlay<'a> {
        PlotPeakOverlay {
            identifications: &self.identifications,
            display_values: &self.display_values,
            energies: &self.energies,
            y_scale,
            log_floor,
        }
    }
}

fn build_peak_overlay(
    samples: &[SampleAnalysis],
    background: Option<&CollapsedSpectrum>,
    subtract_background: bool,
    show_peaks: bool,
    config: &AppConfig,
) -> Option<BuiltPeakOverlay> {
    if !show_peaks {
        return None;
    }
    let params = detection_params_from_config(config);
    let (peaks, energies, display_values) = if subtract_background {
        let sample = samples.first()?;
        let comparison = sample.comparison.as_ref()?;
        let energies = sample.spectrum.energies_kev.clone();
        let counts: Vec<f64> = comparison
            .net_counts
            .iter()
            .map(|value| value.max(0.0))
            .collect();
        let peaks = detect_peaks(&energies, &counts, params);
        (peaks, energies, counts)
    } else if let Some(sample) = samples.first() {
        let peaks = peaks_from_collapsed(&sample.spectrum, params);
        let energies = sample.spectrum.energies_kev.clone();
        let display_values = sample
            .spectrum
            .counts
            .iter()
            .map(|&value| value as f64)
            .collect();
        (peaks, energies, display_values)
    } else {
        let background = background?;
        let peaks = peaks_from_collapsed(background, params);
        let energies = background.energies_kev.clone();
        let display_values = background
            .counts
            .iter()
            .map(|&value| value as f64)
            .collect();
        (peaks, energies, display_values)
    };
    let analysis = analyze_peaks(&peaks, config);
    Some(BuiltPeakOverlay {
        identifications: analysis.identifications,
        sources: analysis.sources,
        display_values,
        energies,
    })
}

fn draw_overlay_plot(
    ui: &mut Ui,
    energies: &[f64],
    width: f64,
    samples: &[SampleAnalysis],
    background: Option<&CollapsedSpectrum>,
    y_scale: YScale,
    smooth_window: usize,
    style: HistogramStyle,
    peak_overlay: Option<&BuiltPeakOverlay>,
) {
    let background_values = background.map(|item| smoothed_background(item, smooth_window));
    let sample_values: Vec<_> = samples
        .iter()
        .map(|sample| smoothed_sample(sample, smooth_window))
        .collect();
    let mut rate_sets: Vec<&[f64]> = background_values
        .as_ref()
        .map(|values| vec![values.as_slice()])
        .unwrap_or_default();
    rate_sets.extend(sample_values.iter().map(|values| values.as_slice()));
    let log_floor = shared_log_floor(&rate_sets);
    let mut owned = Vec::new();
    if let Some(values) = background_values.as_ref() {
        owned.push(owned_series(
            "background",
            energies,
            width,
            values,
            ANALYSIS_BACKGROUND,
            y_scale,
            log_floor,
        ));
    }
    for (index, values) in sample_values.iter().enumerate() {
        owned.push(owned_series(
            &format!("sample_{index}"),
            energies,
            width,
            values,
            analysis_sample_color(index),
            y_scale,
            log_floor,
        ));
    }
    let overlay = peak_overlay.map(|item| item.plot_overlay(y_scale, log_floor));
    show_owned_series(
        ui,
        "analysis_plot",
        &owned,
        y_scale,
        style,
        log_floor,
        overlay.as_ref(),
    );
}

fn draw_net_plot(
    ui: &mut Ui,
    energies: &[f64],
    width: f64,
    samples: &[SampleAnalysis],
    background: Option<&CollapsedSpectrum>,
    y_scale: YScale,
    smooth_window: usize,
    style: HistogramStyle,
    peak_overlay: Option<&BuiltPeakOverlay>,
) {
    let Some(background) = background else {
        ui.label(
            RichText::new("Select a background recording to compute net spectra.").color(MUTED),
        );
        return;
    };
    if samples.is_empty() {
        ui.label(RichText::new("Add one or more sample recordings.").color(MUTED));
        return;
    }
    let net_values: Vec<_> = samples
        .iter()
        .filter_map(|sample| {
            sample
                .comparison
                .as_ref()
                .map(|comparison| smoothed_net(sample, comparison, background, smooth_window))
        })
        .collect();
    if net_values.is_empty() {
        ui.label(RichText::new("No comparable samples for net plot.").color(MUTED));
        return;
    }
    let rate_sets: Vec<&[f64]> = net_values.iter().map(|values| values.as_slice()).collect();
    let log_floor = shared_log_floor(&rate_sets);
    let owned: Vec<_> = net_values
        .iter()
        .enumerate()
        .map(|(index, values)| {
            owned_series(
                &format!("net_{index}"),
                energies,
                width,
                values,
                analysis_sample_color(index),
                y_scale,
                log_floor,
            )
        })
        .collect();
    let overlay = peak_overlay.map(|item| item.plot_overlay(y_scale, log_floor));
    show_owned_series(
        ui,
        "analysis_plot",
        &owned,
        y_scale,
        style,
        log_floor,
        overlay.as_ref(),
    );
}
