use egui::Ui;

use crate::theme::SPACE_SM;

pub fn draw_toolbar(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui)) {
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = SPACE_SM;
        add_contents(ui);
    });
    ui.add_space(SPACE_SM);
}
