use egui::{Sides, Ui};

use crate::theme::SPACE_XS;

pub const TOOLBAR_PLOT_GAP: f32 = 2.0;

pub fn draw_split_plot_toolbar(
    ui: &mut Ui,
    readout: impl FnOnce(&mut Ui),
    controls: impl FnOnce(&mut Ui),
) {
    Sides::new()
        .spacing(SPACE_XS)
        .shrink_left()
        .truncate()
        .show(
            ui,
            |ui| draw_side(ui, readout),
            |ui| draw_side(ui, controls),
        );
    ui.add_space(TOOLBAR_PLOT_GAP);
}

fn draw_side(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui)) {
    ui.spacing_mut().item_spacing.x = SPACE_XS;
    add_contents(ui);
}

pub fn toolbar_height_after(ui: &mut Ui, draw: impl FnOnce(&mut Ui)) -> f32 {
    let top = ui.cursor().top();
    draw(ui);
    (ui.cursor().top() - top).max(1.0)
}
