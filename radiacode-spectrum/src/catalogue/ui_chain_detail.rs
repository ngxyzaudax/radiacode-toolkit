use egui::{RichText, Ui};

use radiacode_nuclides::{
    chain_lines, chain_series, equilibrium_weights, format_half_life, topology_display_name,
};

use crate::app_config::AppConfig;
use crate::catalogue::detail_layout::{DetailLayoutConfig, detail_layout, is_tight_layout};
use crate::catalogue::state::CatalogueState;
use crate::catalogue::ui_chain::draw_decay_chain;
use crate::catalogue::ui_chain_lines::draw_chain_lines;
use crate::catalogue::ui_chain_members::draw_chain_members;
use crate::catalogue::ui_chain_spectrum::draw_chain_spectrum;
use crate::catalogue::ui_chain_stats::draw_chain_stats;
use crate::theme::{MUTED, SPACE_SM};

const PREVIEW_MIN_HEIGHT: f32 = 100.0;
const CHAIN_MIN_HEIGHT: f32 = 160.0;
const SECTION_GAP: f32 = 16.0;
const TIGHT_HEIGHT: f32 = 420.0;
const CHAIN_SHARE: f32 = 0.38;
const CHAIN_MAX_SHARE: f32 = 0.5;

pub fn draw_chain_detail(ui: &mut Ui, state: &mut CatalogueState, config: &mut AppConfig) -> bool {
    let Some(series_index) = state.chains.selected else {
        ui.label(RichText::new("Select a decay chain from the list.").color(MUTED));
        return false;
    };
    let Some(series) = chain_series().get(series_index) else {
        ui.label(RichText::new("Selected chain is not available.").color(MUTED));
        return false;
    };
    let mut changed = false;
    draw_chain_header(ui, series);
    let weights = equilibrium_weights(series);
    let lines = chain_lines(&weights);
    draw_chain_stats(ui, series, &lines, &weights);
    ui.add_space(SPACE_SM);
    let remaining = ui.available_height();
    if is_tight_layout(remaining, TIGHT_HEIGHT) {
        state.chain_collapsed = true;
    }
    let layout = detail_layout(
        remaining,
        state.chain_collapsed,
        DetailLayoutConfig {
            chain_share: CHAIN_SHARE,
            chain_max_share: CHAIN_MAX_SHARE,
            preview_min_height: PREVIEW_MIN_HEIGHT,
            chain_min_height: CHAIN_MIN_HEIGHT,
            section_gap: SECTION_GAP,
        },
    );
    if draw_chain_spectrum(
        ui,
        series,
        &state.chains,
        &mut state.preview_log_scale,
        config,
        layout.preview_height,
    ) {
        changed = true;
    }
    ui.add_space(SPACE_SM);
    draw_chain_members(ui, series, state);
    ui.add_space(SPACE_SM);
    if let Some(id) = draw_chain_lines(ui, series, &mut state.chains) {
        state.mode = crate::catalogue::browse_mode::CatalogueMode::Nuclides;
        state.reveal(id);
    }
    if !state.chain_collapsed {
        ui.add_space(SPACE_SM);
        ui.separator();
        ui.add_space(SPACE_SM);
        draw_decay_chain(
            ui,
            series.head,
            state,
            layout.chain_height.max(CHAIN_MIN_HEIGHT),
        );
    } else {
        ui.horizontal(|ui| {
            if ui.button("Show decay chain").clicked() {
                state.chain_collapsed = false;
            }
        });
    }
    changed
}

fn draw_chain_header(ui: &mut Ui, series: &radiacode_nuclides::ChainSeries) {
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new(&series.name).size(18.0).strong());
        ui.separator();
        ui.label(format!(
            "Head: {}  Family: {}",
            topology_display_name(series.head),
            series.family
        ));
        if let Some(secs) = radiacode_nuclides::topology_half_life_secs(series.head) {
            ui.separator();
            ui.label(format!("Half-life: {}", format_half_life(Some(secs))));
        }
    });
}
