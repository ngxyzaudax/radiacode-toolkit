use egui::{RichText, Ui};

use radiacode_core::DeviceEndpoint;

use crate::theme::{MUTED, SPACE_SM};

use super::icons::paint_transport_icon;
use super::ui_common::draw_section_card;

pub fn draw_connecting(ui: &mut Ui, endpoint: Option<&DeviceEndpoint>) {
    draw_section_card(ui, |ui| {
        ui.vertical_centered(|ui| {
            ui.spinner();
            ui.add_space(SPACE_SM);
            let name = endpoint
                .map(|value| value.address_label())
                .unwrap_or("device");
            ui.label(RichText::new(format!("Connecting to {name}")).strong().size(15.0));
            if let Some(value) = endpoint {
                ui.add_space(SPACE_SM);
                paint_transport_icon(ui, value.transport());
                ui.label(
                    RichText::new(value.address_label())
                        .monospace()
                        .small()
                        .color(MUTED),
                );
            }
            ui.add_space(SPACE_SM);
            ui.label(RichText::new("Keep the detector powered on.").small().color(MUTED));
        });
    });
}
