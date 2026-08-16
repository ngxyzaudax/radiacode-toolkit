use egui::{Button, RichText, Ui, Vec2};

use crate::theme::{MUTED, SPACE_MD, SPACE_XS};

use super::DeviceAction;
use super::ui_common::draw_section_card;

pub fn draw_empty_discovery(
    ui: &mut Ui,
    scanned_once: bool,
    can_scan: bool,
) -> Option<DeviceAction> {
    let mut action = None;
    draw_section_card(ui, |ui| {
        ui.vertical_centered(|ui| {
            if scanned_once {
                ui.label(RichText::new("No detectors found").size(16.0).strong());
            } else {
                ui.label(
                    RichText::new("Searching for detectors…")
                        .size(16.0)
                        .strong(),
                );
            }
            ui.add_space(SPACE_XS);
            if scanned_once {
                draw_hints(ui);
            } else {
                ui.label(
                    RichText::new("Discovery starts automatically when the app opens.")
                        .small()
                        .color(MUTED),
                );
            }
            ui.add_space(SPACE_MD);
            if ui
                .add_enabled(
                    can_scan,
                    Button::new("Scan again").min_size(Vec2::new(140.0, 32.0)),
                )
                .clicked()
            {
                action = Some(DeviceAction::Scan);
            }
        });
    });
    action
}

fn draw_hints(ui: &mut Ui) {
    ui.label(
        RichText::new("Check USB cable is seated")
            .small()
            .color(MUTED),
    );
    ui.add_space(SPACE_XS);
    ui.label(
        RichText::new("Ensure detector is powered on")
            .small()
            .color(MUTED),
    );
    ui.add_space(SPACE_XS);
    ui.label(
        RichText::new("Enable Bluetooth if using wireless")
            .small()
            .color(MUTED),
    );
}
