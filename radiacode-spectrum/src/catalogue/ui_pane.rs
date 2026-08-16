use egui::{RichText, Ui};

use crate::catalogue::browse_mode::{CatalogueMode, draw_mode_toggle};
use crate::catalogue::state::CatalogueState;
use crate::catalogue::ui_chain_list::draw_chain_list;
use crate::catalogue::ui_list::draw_catalogue_list;
use crate::theme::{SPACE_SM, SPACE_XS};

pub enum CatalogueAction {
    FiltersChanged,
}

pub fn draw_catalogue_pane(ui: &mut Ui, state: &mut CatalogueState) -> Option<CatalogueAction> {
    ui.label(RichText::new("Catalogue").strong());
    ui.add_space(SPACE_XS);
    draw_mode_toggle(ui, &mut state.mode);
    ui.add_space(SPACE_XS);
    let changed = match state.mode {
        CatalogueMode::Nuclides => draw_nuclide_filters(ui, state),
        CatalogueMode::Chains => draw_chain_filters(ui, state),
    };
    ui.add_space(SPACE_SM);
    match state.mode {
        CatalogueMode::Nuclides => {
            ui.label(format!("{} nuclides", state.results.len()));
            ui.add_space(SPACE_XS);
            draw_catalogue_list(ui, state);
        }
        CatalogueMode::Chains => {
            ui.label(format!("{} chains", state.chains.results.len()));
            ui.add_space(SPACE_XS);
            draw_chain_list(ui, state);
        }
    }
    if changed {
        Some(CatalogueAction::FiltersChanged)
    } else {
        None
    }
}

fn draw_nuclide_filters(ui: &mut Ui, state: &mut CatalogueState) -> bool {
    ui.add(
        egui::TextEdit::singleline(&mut state.filters.query)
            .hint_text("Search name, element, or mass number"),
    )
    .changed()
}

fn draw_chain_filters(ui: &mut Ui, state: &mut CatalogueState) -> bool {
    ui.add(
        egui::TextEdit::singleline(&mut state.chains.filters.query)
            .hint_text("Search chain, head, or family"),
    )
    .changed()
}
