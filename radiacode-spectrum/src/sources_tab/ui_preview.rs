use egui::{RichText, Ui};

use radiacode_nuclides::{AttributedLine, GammaSource, detectable_lines};

use crate::scale::HistogramStyle;
use crate::sources_tab::signature::{measured_profile, scaled_gamma_lines};
use crate::sources_tab::state::SourcesState;
use crate::sources_tab::ui_spectrum_plot::{
    SpectrumMarker, SpectrumPlotProps, SpectrumSeries, draw_spectrum_plot, spectrum_max_energy,
    spectrum_points,
};
use crate::sources_tab::ui_view::SourcesViewProps;
use crate::synthetic_spectrum::{synthesize, synthesize_grid};
use crate::theme::{ACCENT, MUTED, SPACE_SM, SPECTRUM_BAR};

const GRID_POINTS: usize = 1024;

pub fn draw_source_preview(
    ui: &mut Ui,
    source: &GammaSource,
    state: &mut SourcesState,
    props: SourcesViewProps<'_>,
    plot_height: f32,
) {
    draw_preview_toolbar(ui, state, props);
    ui.add_space(SPACE_SM);
    let lines = detectable_lines(source);
    let gammas = scaled_gamma_lines(lines);
    let max_energy = spectrum_max_energy(&gammas);
    let grid = synthesize_grid(max_energy, GRID_POINTS);
    let values = synthesize(&gammas, props.fwhm_pct, &grid);
    let mut series = vec![SpectrumSeries {
        name: "Expected".to_string(),
        points: spectrum_points(&grid, &values, state.log_scale),
        color: SPECTRUM_BAR,
        style: HistogramStyle::Filled,
    }];
    if let Some(spectrum) = props.measured.filter(|_| state.overlay_measured) {
        let (energies, measured) = measured_profile(spectrum, max_energy);
        series.push(SpectrumSeries {
            name: "Measured".to_string(),
            points: spectrum_points(&energies, &measured, state.log_scale),
            color: ACCENT,
            style: HistogramStyle::Outline,
        });
    }
    let markers = line_markers(lines, state.hovered_line);
    draw_spectrum_plot(
        ui,
        SpectrumPlotProps {
            id: "source_signature_plot",
            height: plot_height,
            max_energy,
            log_scale: state.log_scale,
            series: &series,
            markers: &markers,
            hover_only: false,
        },
    );
}

fn draw_preview_toolbar(ui: &mut Ui, state: &mut SourcesState, props: SourcesViewProps<'_>) {
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new("Expected spectrum").strong().size(14.0));
        ui.add_space(SPACE_SM);
        ui.selectable_value(&mut state.log_scale, false, "Linear");
        ui.selectable_value(&mut state.log_scale, true, "Log");
        ui.separator();
        ui.add_enabled(
            props.measured.is_some(),
            egui::Checkbox::new(&mut state.overlay_measured, "Overlay current spectrum"),
        );
        ui.separator();
        ui.label(
            RichText::new(format!("FWHM {:.1}% at 662 keV", props.fwhm_pct))
                .small()
                .color(MUTED),
        );
    });
}

fn line_markers(lines: &[AttributedLine], hovered: Option<usize>) -> Vec<SpectrumMarker> {
    lines
        .iter()
        .enumerate()
        .filter(|(index, _)| *index == 0 || Some(*index) == hovered)
        .map(|(index, line)| SpectrumMarker {
            energy_kev: line.line.energy_kev,
            label: line.source_name.clone(),
            kind_label: line.line.kind.label().to_string(),
            highlight: Some(index) == hovered,
            strongest: index == 0,
        })
        .collect()
}
