use egui::{RichText, Ui};

use crate::analysis::state::AnalysisState;
use crate::analysis::ui_pane::draw_analysis_library_pane;
use crate::analysis::ui_plot::{AnalysisPlotProps, draw_analysis_plots};
use crate::analysis::ui_toolbar::draw_analysis_toolbar;
use crate::app_config::AppConfig;
use crate::layout::{MasterDetailRegion, draw_master_detail};
use crate::peak_overlay::SpectrumPlotAction;
use crate::scale::{HistogramStyle, YScale};
use crate::theme::{MUTED, analysis_sample_color};

pub fn draw_analysis_view(
    ui: &mut Ui,
    state: &mut AnalysisState,
    y_scale: &mut YScale,
    config: &AppConfig,
) -> Option<SpectrumPlotAction> {
    draw_analysis_toolbar(ui, state, y_scale);
    ui.add_space(8.0);
    let mut plot_action = None;
    let mut pane_open = state.pane_open;
    draw_master_detail(
        ui,
        "analysis_library",
        "Library",
        &mut pane_open,
        |ui, region| match region {
            MasterDetailRegion::Pane => draw_analysis_library_pane(ui, state),
            MasterDetailRegion::Detail => {
                plot_action = draw_analysis_plot(ui, state, *y_scale, config);
            }
        },
    );
    state.pane_open = pane_open;
    plot_action
}

fn draw_analysis_plot(
    ui: &mut Ui,
    state: &mut AnalysisState,
    y_scale: YScale,
    config: &AppConfig,
) -> Option<SpectrumPlotAction> {
    if state.samples.is_empty() && state.background.is_none() {
        ui.label(
            RichText::new(
                "Select one background and one or more sample recordings from the library pane.",
            )
            .color(MUTED),
        );
        return None;
    }
    let style = if state.outline_only {
        HistogramStyle::Outline
    } else {
        HistogramStyle::Filled
    };
    let action = draw_analysis_plots(
        ui,
        AnalysisPlotProps {
            samples: &state.samples,
            background: state.background.as_ref(),
            y_scale,
            smooth_window: state.smooth_window,
            style,
            subtract_background: state.subtract_background,
            show_peaks: state.show_peaks,
            config,
            peak_memo: &mut state.peak_memo,
        },
    );
    if state.subtract_background {
        draw_footer_readouts(ui, state);
    }
    action
}

fn draw_footer_readouts(ui: &mut Ui, state: &AnalysisState) {
    let comparable: Vec<_> = state
        .samples
        .iter()
        .enumerate()
        .filter_map(|(index, sample)| {
            sample
                .comparison
                .as_ref()
                .map(|comparison| (index, sample, comparison))
        })
        .collect();
    if comparable.is_empty() {
        return;
    }
    ui.add_space(12.0);
    ui.separator();
    for (index, sample, comparison) in comparable {
        let color = analysis_sample_color(index);
        let live = sample.spectrum.live_time_secs;
        let net_cps = comparison.net_total / live;
        let net_min_cps = comparison.net_min / live;
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("●").color(color));
            ui.label(RichText::new(&sample.spectrum.name).strong().color(color));
            ui.separator();
            ui.label(format!("T_S/T_B: {:.4}", comparison.scale_factor));
            ui.separator();
            ui.label(format!("Net total: {net_cps:.2} cps"));
            ui.separator();
            ui.label(format!("Neg bins: {}", comparison.negative_bin_count));
            ui.separator();
            ui.label(format!("Net min: {net_min_cps:.2} cps"));
        });
    }
}
