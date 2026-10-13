use egui::{RichText, Ui};

use radiacode_nuclides::{EnergyQuery, match_nuclides_at_energy, match_sources_at_energy};

use crate::layout::page_scroll;
use crate::peak_id::action::PeakIdAction;
use crate::peak_id::state::PeakIdState;
use crate::peak_id::ui_input::draw_energy_input;
use crate::peak_id::ui_nuclide_matches::draw_nuclide_matches;
use crate::peak_id::ui_source_matches::draw_source_matches;
use crate::theme::{MUTED, SPACE_LG, SPACE_SM};

#[derive(Clone, Copy)]
pub struct PeakIdProps<'a> {
    pub fwhm_pct: f64,
    pub min_supported_half_life_secs: f64,
    pub peaks_kev: &'a [f64],
}

pub fn draw_peak_id_view(
    ui: &mut Ui,
    state: &mut PeakIdState,
    props: PeakIdProps<'_>,
) -> Option<PeakIdAction> {
    let mut action = None;
    page_scroll(ui, "peak_id_page", |ui| {
        draw_intro(ui);
        draw_energy_input(ui, state, props);
        ui.add_space(SPACE_LG);
        let query = EnergyQuery {
            energy_kev: state.energy_kev,
            fwhm_pct: props.fwhm_pct,
            peaks_kev: props.peaks_kev,
        };
        action = draw_source_matches(ui, &match_sources_at_energy(query));
        ui.add_space(SPACE_LG);
        ui.checkbox(
            &mut state.include_all_nuclides,
            "Also search all long-lived nuclides",
        );
        if state.include_all_nuclides {
            let matches = match_nuclides_at_energy(
                state.energy_kev,
                props.fwhm_pct,
                props.min_supported_half_life_secs,
            );
            draw_nuclide_matches(ui, &matches);
        }
    });
    action
}

fn draw_intro(ui: &mut Ui) {
    ui.label(RichText::new("Peak ID").strong().size(18.0));
    ui.label(
        RichText::new(
            "Enter a peak energy to list gamma sources with a line inside the detector resolution.",
        )
        .color(MUTED),
    );
    ui.add_space(SPACE_SM);
}
