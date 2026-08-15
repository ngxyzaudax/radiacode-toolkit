use egui::Ui;

use crate::layout::draw_toolbar;
use crate::settings::state::{SettingsSection, SettingsState};

pub fn draw_settings_nav(ui: &mut Ui, state: &mut SettingsState) {
    draw_toolbar(ui, |ui| {
        ui.selectable_value(&mut state.section, SettingsSection::Device, "Device");
        ui.selectable_value(
            &mut state.section,
            SettingsSection::Application,
            "Application",
        );
    });
}
