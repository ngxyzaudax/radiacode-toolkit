#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CatalogueMode {
    #[default]
    Nuclides,
    Chains,
}

pub fn draw_mode_toggle(ui: &mut egui::Ui, mode: &mut CatalogueMode) {
    ui.horizontal(|ui| {
        ui.selectable_value(mode, CatalogueMode::Nuclides, "Nuclides");
        ui.selectable_value(mode, CatalogueMode::Chains, "Chains");
    });
}
