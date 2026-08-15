use egui::{RichText, Ui, Vec2b};
use egui_plot::{Plot, PlotPoint, PlotPoints, Points};

use radiacode_nuclides::{GammaLine, Nuclide, strongest_gamma};

use crate::app_config::AppConfig;
use crate::catalogue::state::CatalogueState;
use crate::plot_style::styled_line;
use crate::scale::HistogramStyle;
use crate::scale::{YScale, display_value, y_axis_top};
use crate::synthetic_spectrum::{synthesize, synthesize_grid};
use crate::theme::{ACCENT, MUTED, SPECTRUM_BAR, SPACE_SM};

const GRID_POINTS: usize = 1024;

pub fn draw_peak_preview(
    ui: &mut Ui,
    nuclide: &Nuclide,
    state: &mut CatalogueState,
    config: &mut AppConfig,
    plot_height: f32,
) -> bool {
    let mut changed = false;
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new("Gamma spectrum").strong().size(14.0));
        ui.add_space(SPACE_SM);
        ui.label(RichText::new("FWHM @ 662 keV").size(12.0).color(MUTED));
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
        ui.selectable_value(&mut state.preview_log_scale, false, "Linear");
        ui.selectable_value(&mut state.preview_log_scale, true, "Log");
    });
    ui.add_space(SPACE_SM);
    let max_energy = preview_max_energy(&nuclide.gammas);
    let grid = synthesize_grid(max_energy, GRID_POINTS);
    let values = synthesize(&nuclide.gammas, config.catalogue_fwhm_pct, &grid);
    let y_scale = if state.preview_log_scale {
        YScale::Logarithmic
    } else {
        YScale::Linear
    };
    let points: Vec<[f64; 2]> = grid
        .iter()
        .zip(values.iter())
        .map(|(&energy, &value)| [energy, display_value(value, y_scale)])
        .collect();
    let peak = points.iter().map(|point| point[1]).fold(0.0_f64, f64::max);
    let y_top = y_axis_top(peak, y_scale);
    let highlight = state
        .hovered_gamma
        .and_then(|index| nuclide.gammas.get(index));
    Plot::new("catalogue_peak_preview")
        .height(plot_height)
        .allow_zoom(false)
        .allow_drag(false)
        .allow_scroll(false)
        .auto_bounds(Vec2b::new(false, false))
        .include_y(0.0)
        .x_axis_label("Energy (keV)")
        .y_axis_label(if state.preview_log_scale {
            "Relative γ intensity (log10)"
        } else {
            "Relative γ intensity"
        })
        .show(ui, |plot_ui| {
            plot_ui.set_plot_bounds_x(0.0..=max_energy);
            plot_ui.set_plot_bounds_y(0.0..=y_top);
            plot_ui.line(styled_line(
                "preview",
                points,
                SPECTRUM_BAR,
                HistogramStyle::Filled,
            ));
            draw_line_markers(plot_ui, nuclide, highlight);
        });
    changed
}

fn preview_max_energy(gammas: &[GammaLine]) -> f64 {
    gammas
        .iter()
        .map(|gamma| gamma.energy_kev)
        .fold(400.0_f64, |max, energy| max.max(energy))
        * 1.1
}

fn draw_line_markers(
    plot_ui: &mut egui_plot::PlotUi,
    nuclide: &Nuclide,
    highlight: Option<&GammaLine>,
) {
    let strongest = strongest_gamma(&nuclide.gammas);
    for gamma in &nuclide.gammas {
        let is_highlight = highlight.is_some_and(|entry| entry.energy_kev == gamma.energy_kev);
        let is_strong = strongest.is_some_and(|entry| entry.energy_kev == gamma.energy_kev);
        if !is_highlight && !is_strong {
            continue;
        }
        let color = if is_highlight { ACCENT } else { MUTED };
        plot_ui.vline(
            egui_plot::VLine::new(
                format!("line_{:.1}", gamma.energy_kev),
                gamma.energy_kev,
            )
            .color(color)
            .width(if is_highlight { 2.0 } else { 1.0 }),
        );
        if is_strong || is_highlight {
            plot_ui.points(
                Points::new(
                    "marker",
                    PlotPoints::from(vec![[gamma.energy_kev, 0.0]]),
                )
                .color(color)
                .radius(3.0),
            );
        }
        if is_strong {
            plot_ui.text(
                egui_plot::Text::new(
                    format!("label_{:.1}", gamma.energy_kev),
                    PlotPoint::new(gamma.energy_kev, 0.0),
                    format!("{:.1} keV {}", gamma.energy_kev, gamma.kind.label()),
                )
                .color(color),
            );
        }
    }
}
