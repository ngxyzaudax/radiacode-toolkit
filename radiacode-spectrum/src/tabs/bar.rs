use egui::Ui;

use crate::view_tab::ViewTab;

pub fn draw_tab_bar(ui: &mut Ui, active: ViewTab) -> Option<ViewTab> {
    let mut selected = active;
    ui.horizontal_wrapped(|ui| {
        ui.selectable_value(&mut selected, ViewTab::Device, ViewTab::Device.label());
        ui.selectable_value(&mut selected, ViewTab::Monitor, ViewTab::Monitor.label());
        ui.selectable_value(&mut selected, ViewTab::Spectrum, ViewTab::Spectrum.label());
        ui.selectable_value(&mut selected, ViewTab::Spectrogram, ViewTab::Spectrogram.label());
        ui.selectable_value(&mut selected, ViewTab::Analysis, ViewTab::Analysis.label());
        ui.selectable_value(&mut selected, ViewTab::Catalogue, ViewTab::Catalogue.label());
        ui.selectable_value(&mut selected, ViewTab::Settings, ViewTab::Settings.label());
        ui.selectable_value(&mut selected, ViewTab::About, ViewTab::About.label());
    });
    if selected != active {
        Some(selected)
    } else {
        None
    }
}
