use egui::Ui;

use radiacode_nuclides::{Nuclide, strongest_gamma};

use crate::app_config::AppConfig;
use crate::catalogue::state::CatalogueState;
use crate::catalogue::ui_spectrum_plot::{
    SpectrumMarker, SpectrumPlotProps, SpectrumSeries, draw_spectrum_plot, spectrum_max_energy,
    spectrum_points,
};
use crate::scale::HistogramStyle;
use crate::synthetic_spectrum::{synthesize, synthesize_grid};
use crate::theme::{SPACE_SM, SPECTRUM_BAR};
use crate::ui::widgets::{clamp_spectrum_fwhm, draw_spectrum_scale_toolbar};

const GRID_POINTS: usize = 1024;

pub fn draw_peak_preview(
    ui: &mut Ui,
    nuclide: &Nuclide,
    state: &mut CatalogueState,
    config: &mut AppConfig,
    plot_height: f32,
) -> bool {
    let changed = draw_spectrum_scale_toolbar(
        ui,
        "Gamma spectrum",
        &mut config.catalogue_fwhm_pct,
        &mut state.preview_log_scale,
    );
    if changed {
        clamp_spectrum_fwhm(config);
    }
    ui.add_space(SPACE_SM);
    let max_energy = spectrum_max_energy(&nuclide.gammas);
    let grid = synthesize_grid(max_energy, GRID_POINTS);
    let values = synthesize(&nuclide.gammas, config.catalogue_fwhm_pct, &grid);
    let points = spectrum_points(&grid, &values, state.preview_log_scale);
    let highlight = state
        .hovered_gamma
        .and_then(|index| nuclide.gammas.get(index));
    let markers = nuclide_markers(nuclide, highlight);
    let series = [SpectrumSeries {
        name: "preview".to_string(),
        points,
        color: SPECTRUM_BAR,
        style: HistogramStyle::Filled,
    }];
    draw_spectrum_plot(
        ui,
        SpectrumPlotProps {
            id: "catalogue_peak_preview",
            height: plot_height,
            max_energy,
            log_scale: state.preview_log_scale,
            series: &series,
            markers: &markers,
            hover_only: false,
        },
    );
    changed
}

fn nuclide_markers(
    nuclide: &Nuclide,
    highlight: Option<&radiacode_nuclides::GammaLine>,
) -> Vec<SpectrumMarker> {
    let strongest = strongest_gamma(&nuclide.gammas);
    nuclide
        .gammas
        .iter()
        .filter_map(|gamma| {
            let is_highlight = highlight.is_some_and(|entry| entry.energy_kev == gamma.energy_kev);
            let is_strong = strongest.is_some_and(|entry| entry.energy_kev == gamma.energy_kev);
            if !is_highlight && !is_strong {
                return None;
            }
            Some(SpectrumMarker {
                energy_kev: gamma.energy_kev,
                label: String::new(),
                kind_label: gamma.kind.label().to_string(),
                highlight: is_highlight,
                strongest: is_strong,
            })
        })
        .collect()
}
