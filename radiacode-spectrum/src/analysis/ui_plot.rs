use egui::RichText;
use egui::Ui;

use crate::analysis::spectrum::CollapsedSpectrum;
use crate::analysis::state::SampleAnalysis;
use crate::analysis::ui_plot_bars::{PlotPeakOverlay, owned_series, shared_log_floor, show_owned_series};
use crate::analysis::ui_plot_legend::draw_legend;
use crate::analysis::ui_plot_values::{peak_source_values, smoothed_background, smoothed_net, smoothed_sample};
use crate::app_config::AppConfig;
use crate::identify::identify_peaks;
use crate::peak_detect::SpectrumPeak;
use crate::peak_overlay::{SpectrumPlotAction, draw_identification_chips};
use crate::peak_profile::peaks_from_values;
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
    pub identify_isotopes: bool,
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
        .map(|s| &s.spectrum)
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
        props.smooth_window,
        props.show_peaks,
        props.identify_isotopes,
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
    if let Some(overlay) = peak_overlay {
        if let Some(identifications) = overlay.identifications.as_ref() {
            return draw_identification_chips(ui, identifications);
        }
    }
    None
}

struct BuiltPeakOverlay {
    peaks: Vec<SpectrumPeak>,
    identifications: Option<Vec<radiacode_nuclides::PeakIdentification>>,
}

impl BuiltPeakOverlay {
    fn as_plot_overlay(&self, identify: bool) -> PlotPeakOverlay<'_> {
        PlotPeakOverlay {
            peaks: &self.peaks,
            identifications: if identify {
                self.identifications.as_deref()
            } else {
                None
            },
        }
    }
}

fn build_peak_overlay(
    samples: &[SampleAnalysis],
    background: Option<&CollapsedSpectrum>,
    subtract_background: bool,
    smooth_window: usize,
    show_peaks: bool,
    identify_isotopes: bool,
    config: &AppConfig,
) -> Option<BuiltPeakOverlay> {
    if !show_peaks {
        return None;
    }
    let (energies, values) =
        peak_source_values(samples, background, subtract_background, smooth_window)?;
    let peaks = peaks_from_values(&energies, &values, smooth_window);
    let identifications = if identify_isotopes {
        Some(identify_peaks(&peaks, config))
    } else {
        None
    };
    Some(BuiltPeakOverlay {
        peaks,
        identifications,
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
    let overlay = peak_overlay.map(|item| item.as_plot_overlay(item.identifications.is_some()));
    show_owned_series(
        ui,
        "analysis_plot",
        &owned,
        y_scale,
        style,
        log_floor,
        overlay,
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
    let overlay = peak_overlay.map(|item| item.as_plot_overlay(item.identifications.is_some()));
    show_owned_series(
        ui,
        "analysis_plot",
        &owned,
        y_scale,
        style,
        log_floor,
        overlay,
    );
}
