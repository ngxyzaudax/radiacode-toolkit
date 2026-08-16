use egui::{Button, Color32, RichText, Ui, Vec2};

use super::MonitorLinkHealth;
use super::refresh_age::refresh_age_label;
use crate::model::DeviceInfo;
use crate::theme::{MUTED, SPACE_MD, SPACE_SM};
use crate::ui_device_status::draw_status_row;

use super::DeviceAction;
use super::icons::paint_transport_icon;
use super::ui_common::draw_section_card;

const DANGER: Color32 = Color32::from_rgb(220, 90, 90);
const WARN: Color32 = Color32::from_rgb(230, 170, 70);

pub fn draw_connected(
    ui: &mut Ui,
    info: &DeviceInfo,
    link_health: MonitorLinkHealth,
    last_spectrum_fetch: Option<std::time::Instant>,
    last_monitor_fetch: Option<std::time::Instant>,
) -> Option<DeviceAction> {
    let mut action = None;
    draw_section_card(ui, |ui| {
        ui.horizontal(|ui| {
            paint_transport_icon(ui, info.transport);
            ui.add_space(SPACE_SM);
            ui.vertical(|ui| {
                ui.label(
                    RichText::new(format!("RadiaCode {}", info.model))
                        .size(18.0)
                        .strong(),
                );
                ui.label(RichText::new(&info.serial).monospace().size(20.0));
            });
        });
        ui.add_space(SPACE_MD);
        draw_status_row(ui, info);
        ui.add_space(SPACE_SM);
        draw_refresh_ages(ui, last_spectrum_fetch, last_monitor_fetch);
        draw_link_health(ui, link_health);
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
            Button::new(RichText::new("Disconnect").color(DANGER)).min_size(Vec2::new(160.0, 34.0)),
        )
        .clicked()
    {
        action = Some(DeviceAction::Disconnect);
    }
    action
}

fn draw_refresh_ages(
    ui: &mut Ui,
    last_spectrum_fetch: Option<std::time::Instant>,
    last_monitor_fetch: Option<std::time::Instant>,
) {
    ui.label(
        RichText::new(format!(
            "Spectrum {} · Monitor {}",
            refresh_age_label(last_spectrum_fetch),
            refresh_age_label(last_monitor_fetch),
        ))
        .small()
        .color(MUTED),
    );
}

fn draw_link_health(ui: &mut Ui, link_health: MonitorLinkHealth) {
    let color = if link_health.has_issues() {
        WARN
    } else {
        MUTED
    };
    ui.label(RichText::new(link_health.summary()).small().color(color));
}
