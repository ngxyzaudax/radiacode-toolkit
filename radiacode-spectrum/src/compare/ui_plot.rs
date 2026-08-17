use egui::Ui;

use crate::app_config::AppConfig;
use crate::compare::peak_overlay_build::build_peak_overlay;
use crate::compare::spectrum::CollapsedSpectrum;
use crate::compare::state::ComparedSample;
use crate::compare::ui_plot_legend::draw_legend;
use crate::compare::ui_plot_net::draw_net_plot;
use crate::compare::ui_plot_overlay::{SpectrumPlotDrawParams, draw_overlay_plot};
use crate::peak_overlay::{SpectrumPlotAction, draw_source_chips};
use crate::scale::{HistogramStyle, YScale};
use crate::theme::MUTED;
use egui::RichText;

pub struct ComparePlotProps<'a> {
    pub samples: &'a [ComparedSample],
    pub background: Option<&'a CollapsedSpectrum>,
    pub y_scale: YScale,
    pub smooth_window: usize,
    pub style: HistogramStyle,
    pub subtract_background: bool,
    pub show_peaks: bool,
    pub config: &'a AppConfig,
    pub peak_memo: &'a mut crate::peaks::PeakMemo,
}

pub fn draw_compare_plots(ui: &mut Ui, props: ComparePlotProps<'_>) -> Option<SpectrumPlotAction> {
    if props.samples.is_empty() && props.background.is_none() {
        return None;
    }
    let axis = props
        .samples
        .first()
        .map(|sample| &sample.spectrum)
        .or(props.background)?;
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
        props.smooth_window,
        props.config,
        props.peak_memo,
    );
    if props.show_peaks
        && let Some(overlay) = peak_overlay.as_ref()
    {
        ui.label(
            RichText::new(format!("{} detected peaks", overlay.peak_count))
                .small()
                .color(MUTED),
        );
    }
    let draw_params = SpectrumPlotDrawParams {
        energies,
        width,
        samples: props.samples,
        background: props.background,
        y_scale: props.y_scale,
        smooth_window: props.smooth_window,
        style: props.style,
        peak_overlay: peak_overlay.as_ref(),
    };
    if show_net {
        draw_net_plot(ui, draw_params);
    } else {
        draw_overlay_plot(ui, draw_params);
    }
    peak_overlay.and_then(|overlay| draw_source_chips(ui, &overlay.sources))
}
