use egui::{RichText, Ui};

use radiacode_nuclides::{Nuclide, format_half_life, nuclide_by_id, series_for_member};

use crate::app_config::AppConfig;
use crate::catalogue::state::CatalogueState;
use crate::catalogue::ui_chain::draw_decay_chain;
use crate::catalogue::ui_preview::draw_peak_preview;
use crate::catalogue::ui_stats::draw_nuclide_stats;
use crate::layout::safe_span;
use crate::theme::{MUTED, SPACE_SM};

const PREVIEW_MIN_HEIGHT: f32 = 100.0;
const CHAIN_MIN_HEIGHT: f32 = 160.0;
const SECTION_GAP: f32 = 16.0;
const TIGHT_HEIGHT: f32 = 380.0;
const CHAIN_SHARE: f32 = 0.48;
const CHAIN_MAX_SHARE: f32 = 0.58;

pub fn draw_catalogue_detail(
    ui: &mut Ui,
    state: &mut CatalogueState,
    config: &mut AppConfig,
) -> bool {
    let Some(id) = state.selected else {
        ui.label(RichText::new("Select a nuclide from the list.").color(MUTED));
        return false;
    };
    let Some(nuclide) = nuclide_by_id(id) else {
        ui.label(RichText::new("Selected nuclide is not in the catalogue.").color(MUTED));
        return false;
    };
    draw_nuclide_header(ui, nuclide);
    if let Some(series) = series_for_member(id) {
        ui.horizontal(|ui| {
            if ui
                .button(format!("View in {} series", series.name))
                .clicked()
            {
                state.select_chain_by_head(series.head);
            }
        });
        ui.add_space(SPACE_SM);
    }
    draw_nuclide_stats(ui, nuclide);
    ui.add_space(SPACE_SM);
    let remaining = ui.available_height();
    let tight = remaining < TIGHT_HEIGHT;
    if tight {
        state.chain_collapsed = true;
    }
    let chain_fraction = if state.chain_collapsed {
        0.0
    } else {
        CHAIN_SHARE
    };
    let chain_height = if state.chain_collapsed {
        0.0
    } else {
        safe_span(remaining * chain_fraction, 0.0, CHAIN_MIN_HEIGHT).min(remaining * CHAIN_MAX_SHARE)
    };
    let preview_height = safe_span(
        remaining - chain_height - if state.chain_collapsed { 0.0 } else { SECTION_GAP },
        0.0,
        PREVIEW_MIN_HEIGHT,
    );
    let changed = draw_peak_preview(ui, nuclide, state, config, preview_height);
    if !state.chain_collapsed {
        ui.add_space(SPACE_SM);
        ui.separator();
        ui.add_space(SPACE_SM);
        draw_decay_chain(ui, id, state, chain_height.max(CHAIN_MIN_HEIGHT));
    } else {
        ui.horizontal(|ui| {
            if ui.button("Show decay chain").clicked() {
                state.chain_collapsed = false;
            }
        });
    }
    changed
}

fn draw_nuclide_header(ui: &mut Ui, nuclide: &Nuclide) {
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new(&nuclide.display_name).size(18.0).strong());
        ui.separator();
        ui.label(format!(
            "Z={} N={}  Half-life: {}",
            nuclide.id.z,
            nuclide.id.n,
            format_half_life(nuclide.half_life_secs)
        ));
    });
}
