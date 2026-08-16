use egui::{RichText, Ui};

use radiacode_nuclides::{
    AttributedLine, ChainSeries, NuclideId, chain_lines, equilibrium_weights,
};

use crate::app_config::AppConfig;
use crate::catalogue::chain_state::ChainBrowseState;
use crate::catalogue::ui_spectrum_plot::{
    SpectrumPlotProps, SpectrumSeries, draw_spectrum_plot, spectrum_max_energy, spectrum_points,
};
use crate::scale::HistogramStyle;
use crate::synthetic_spectrum::{synthesize, synthesize_grid};
use crate::theme::{SPECTRUM_BAR, SPACE_SM};

const GRID_POINTS: usize = 1024;

pub fn draw_chain_spectrum(
    ui: &mut Ui,
    series: &ChainSeries,
    chains: &ChainBrowseState,
    preview_log_scale: &mut bool,
    config: &mut AppConfig,
    plot_height: f32,
) -> bool {
    let changed = draw_spectrum_toolbar(ui, preview_log_scale, config);
    ui.add_space(SPACE_SM);
    let weights = equilibrium_weights(series);
    let lines = chain_lines(&weights);
    let gammas = lines
        .iter()
        .map(|entry| entry.line.clone())
        .collect::<Vec<_>>();
    let max_energy = spectrum_max_energy(&gammas);
    let grid = synthesize_grid(max_energy, GRID_POINTS);
    let total_values = synthesize(&gammas, config.catalogue_fwhm_pct, &grid);
    let total_points = spectrum_points(&grid, &total_values, *preview_log_scale);
    let plot_series = [SpectrumSeries {
        name: "total".to_string(),
        points: total_points,
        color: SPECTRUM_BAR,
        style: HistogramStyle::Filled,
    }];
    let markers = chain_markers(&lines, chains.hovered_member);
    draw_spectrum_plot(
        ui,
        SpectrumPlotProps {
            id: "catalogue_chain_preview",
            height: plot_height,
            max_energy,
            log_scale: *preview_log_scale,
            series: &plot_series,
            markers: &markers,
            hover_only: true,
        },
    );
    changed
}

fn draw_spectrum_toolbar(
    ui: &mut Ui,
    preview_log_scale: &mut bool,
    config: &mut AppConfig,
) -> bool {
    let mut changed = false;
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new("Chain spectrum").strong().size(14.0));
        ui.add_space(SPACE_SM);
        ui.label(RichText::new("FWHM @ 662 keV").size(12.0).color(crate::theme::MUTED));
        if ui
            .add(
                egui::Slider::new(&mut config.catalogue_fwhm_pct, 1.0..=20.0)
                    .suffix("%")
                    .fixed_decimals(1),
            )
            .changed()
        {
            config.clamp();
            changed = true;
        }
        ui.separator();
        ui.selectable_value(preview_log_scale, false, "Linear");
        ui.selectable_value(preview_log_scale, true, "Log");
    });
    changed
}

fn chain_markers(
    lines: &[AttributedLine],
    hovered_member: Option<NuclideId>,
) -> Vec<crate::catalogue::ui_spectrum_plot::SpectrumMarker> {
    let Some(member) = hovered_member else {
        return Vec::new();
    };
    lines
        .iter()
        .filter(|line| line.source == member)
        .map(|line| crate::catalogue::ui_spectrum_plot::SpectrumMarker {
            energy_kev: line.line.energy_kev,
            label: line.source_name.clone(),
            kind_label: line.line.kind.label().to_string(),
            highlight: true,
            strongest: false,
        })
        .collect()
}
