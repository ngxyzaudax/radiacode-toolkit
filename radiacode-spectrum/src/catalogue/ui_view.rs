use egui::{RichText, Ui};

use radiacode_nuclides::nuclide_count;

use crate::app_config::AppConfig;
use crate::catalogue::browse_mode::CatalogueMode;
use crate::catalogue::state::CatalogueState;
use crate::catalogue::ui_chain_detail::draw_chain_detail;
use crate::catalogue::ui_detail::draw_catalogue_detail;
use crate::catalogue::ui_pane::{CatalogueAction, draw_catalogue_pane};
use crate::layout::{MasterDetailRegion, draw_master_detail};
use crate::theme::{MUTED, SPACE_MD};

const FOOTER_RESERVE: f32 = 28.0;

pub fn draw_catalogue_view(
    ui: &mut Ui,
    state: &mut CatalogueState,
    config: &mut AppConfig,
) -> bool {
    if nuclide_count() == 0 {
        ui.label(
            RichText::new("Nuclide catalogue is empty. Regenerate data/nuclides.json.")
                .color(MUTED),
        );
        return false;
    }
    let body_height = (ui.available_height() - FOOTER_RESERVE - SPACE_MD).max(160.0);
    ui.set_min_height(body_height);
    let mut changed = false;
    let mut pane_open = state.pane_open;
    draw_master_detail(
        ui,
        "catalogue_list",
        "Nuclides",
        &mut pane_open,
        |ui, region| match region {
            MasterDetailRegion::Pane => {
                if let Some(CatalogueAction::FiltersChanged) = draw_catalogue_pane(ui, state) {
                    match state.mode {
                        CatalogueMode::Nuclides => state.refresh_results(),
                        CatalogueMode::Chains => state.chains.refresh_results(),
                    }
                }
            }
            MasterDetailRegion::Detail => {
                let detail_changed = match state.mode {
                    CatalogueMode::Nuclides => draw_catalogue_detail(ui, state, config),
                    CatalogueMode::Chains => draw_chain_detail(ui, state, config),
                };
                if detail_changed {
                    changed = true;
                }
            }
        },
    );
    state.pane_open = pane_open;
    ui.add_space(SPACE_MD);
    ui.label(
        RichText::new("Nuclear data from IAEA Livechart / ENSDF.")
            .small()
            .color(MUTED),
    );
    changed
}
