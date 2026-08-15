use egui::{RichText, Ui};

use crate::catalogue::state::CatalogueState;
use crate::catalogue::ui_list::draw_catalogue_list;
use crate::theme::{SPACE_SM, SPACE_XS};

pub enum CatalogueAction {
    FiltersChanged,
}

pub fn draw_catalogue_pane(ui: &mut Ui, state: &mut CatalogueState) -> Option<CatalogueAction> {
    ui.label(RichText::new("Catalogue").strong());
    ui.add_space(SPACE_XS);
    let changed = ui
        .add(
            egui::TextEdit::singleline(&mut state.filters.query)
                .hint_text("Search name, element, or mass number"),
        )
        .changed();
    ui.add_space(SPACE_SM);
    ui.label(format!("{} nuclides", state.results.len()));
    ui.add_space(SPACE_XS);
    draw_catalogue_list(ui, state);
    if changed {
        Some(CatalogueAction::FiltersChanged)
    } else {
        None
    }
}
