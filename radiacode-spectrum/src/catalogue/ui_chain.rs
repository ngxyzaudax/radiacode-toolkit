use egui::{RichText, Ui};

use radiacode_nuclides::NuclideId;

use crate::catalogue::state::CatalogueState;
use crate::catalogue::ui_chain_viewport::draw_chain_viewport;

pub fn draw_decay_chain(
    ui: &mut Ui,
    focus: NuclideId,
    state: &mut CatalogueState,
    viewport_height: f32,
) {
    ui.label(RichText::new("Decay chain").strong().size(14.0));
    ui.add_space(4.0);
    draw_chain_viewport(ui, focus, state, viewport_height.max(100.0));
}
