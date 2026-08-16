use egui::{Button, Color32, RichText, Ui, Vec2};

use crate::model::DeviceInfo;
use crate::theme::{MUTED, SPACE_MD, SPACE_SM};
use crate::ui_device_status::draw_status_row;

use super::icons::paint_transport_icon;
use super::ui_common::draw_section_card;
use super::DeviceAction;

const DANGER: Color32 = Color32::from_rgb(220, 90, 90);

pub fn draw_connected(ui: &mut Ui, info: &DeviceInfo) -> Option<DeviceAction> {
    let mut action = None;
    draw_section_card(ui, |ui| {
        ui.horizontal(|ui| {
            paint_transport_icon(ui, info.transport);
            ui.add_space(SPACE_SM);
            ui.vertical(|ui| {
                ui.label(RichText::new(format!("RadiaCode {}", info.model)).size(18.0).strong());
                ui.label(RichText::new(&info.serial).monospace().size(20.0));
            });
        });
        ui.add_space(SPACE_MD);
        draw_status_row(ui, info);
        ui.add_space(SPACE_SM);
        ui.label(
            RichText::new(format!(
                "Firmware {} · {} · {}",
                info.firmware,
                info.transport_label(),
                info.address
            ))
            .small()
            .color(MUTED),
        );
    });
    ui.add_space(SPACE_MD);
    if ui
        .add(
            Button::new(RichText::new("Disconnect").color(DANGER))
                .min_size(Vec2::new(160.0, 34.0)),
        )
        .clicked()
    {
        action = Some(DeviceAction::Disconnect);
    }
    action
}
