use egui::{RichText, Ui};

use crate::model::DeviceInfo;
use crate::theme::ACCENT;
use crate::ui_device_status::draw_status_row;

use super::DeviceAction;

pub fn draw_connected(ui: &mut Ui, info: &DeviceInfo) -> Option<DeviceAction> {
    let mut action = None;
    ui.label(RichText::new("Connected").size(20.0).color(ACCENT).strong());
    ui.add_space(12.0);
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(RichText::new(format!("Model {}", info.model)).size(16.0));
            ui.label(RichText::new(&info.serial).size(28.0).strong());
        });
    });
    ui.add_space(12.0);
    draw_status_row(ui, info);
    ui.add_space(12.0);
    ui.separator();
    ui.add_space(8.0);
    ui.label(format!("Transport {}", info.transport_label()));
    ui.label(RichText::new(&info.address).monospace());
    ui.label(format!("Firmware {}", info.firmware));
    ui.label(format!(
        "Calibration  a0={:.2}  a1={:.3}  a2={:.5}",
        info.energy_calib[0], info.energy_calib[1], info.energy_calib[2]
    ));
    ui.add_space(16.0);
    if ui
        .add(egui::Button::new("Disconnect").min_size([200.0, 36.0].into()))
        .clicked()
    {
        action = Some(DeviceAction::Disconnect);
    }
    action
}
