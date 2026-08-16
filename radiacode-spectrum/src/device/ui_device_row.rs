use egui::{Button, Color32, RichText, Sense, Ui, Vec2};

use radiacode_core::{DeviceEndpoint, DiscoveredDevice, TransportKind};

use crate::theme::{ACCENT, MUTED, SPACE_SM, SPACE_XS};
use crate::ui_device_status::{paint_signal_icon, signal_color};

use super::icons::paint_transport_icon;
use super::ui_common::{draw_accent_card, draw_muted_card};
use super::DeviceAction;

const ROW_HEIGHT: f32 = 44.0;
const HOVER_FILL: Color32 = Color32::from_rgb(36, 40, 48);

pub fn draw_reconnect_card(
    ui: &mut Ui,
    endpoint: &DeviceEndpoint,
    device: Option<&DiscoveredDevice>,
    busy: bool,
) -> Option<DeviceAction> {
    let detected = device.is_some();
    let label = device
        .map(DiscoveredDevice::display_label)
        .unwrap_or_else(|| "Last device".to_string());
    let mut action = None;
    let address = endpoint.address_label().to_string();
    let transport = endpoint.transport();
    let endpoint = endpoint.clone();
    let draw_card = |ui: &mut Ui| {
        ui.horizontal(|ui| {
            paint_transport_icon(ui, transport);
            ui.add_space(SPACE_SM);
            ui.vertical(|ui| {
                ui.label(RichText::new(&label).strong().size(15.0));
                ui.label(RichText::new(&address).monospace().small().color(MUTED));
                if !detected {
                    ui.label(RichText::new("Not detected").small().color(MUTED));
                }
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let button = Button::new("Reconnect")
                    .min_size(Vec2::new(108.0, 32.0))
                    .fill(ACCENT.gamma_multiply(0.35));
                if ui.add_enabled(detected && !busy, button).clicked() {
                    action = Some(DeviceAction::Connect(endpoint.clone()));
                }
            });
        });
    };
    if detected {
        draw_accent_card(ui, draw_card);
    } else {
        draw_muted_card(ui, draw_card);
    }
    action
}

pub fn draw_device_row(
    ui: &mut Ui,
    device: &DiscoveredDevice,
    busy: bool,
) -> Option<DeviceAction> {
    let mut action = None;
    let width = ui.available_width();
    let (rect, response) =
        ui.allocate_exact_size(Vec2::new(width, ROW_HEIGHT), Sense::click());
    let fill = if response.hovered() && !busy {
        HOVER_FILL
    } else {
        Color32::TRANSPARENT
    };
    ui.painter().rect_filled(rect, 0.0, fill);
    ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
        ui.horizontal(|ui| {
            paint_transport_icon(ui, device.endpoint.transport());
            ui.add_space(SPACE_SM);
            ui.vertical(|ui| {
                ui.label(RichText::new(device.display_label()).strong().size(14.0));
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(device.endpoint.address_label())
                            .monospace()
                            .small()
                            .color(MUTED),
                    );
                    if device.endpoint.transport() == TransportKind::Bluetooth {
                        if let Some(rssi) = device.rssi {
                            ui.add_space(SPACE_XS);
                            paint_signal_icon(ui, rssi);
                            ui.label(
                                RichText::new(format!("{rssi} dBm"))
                                    .small()
                                    .color(signal_color(rssi)),
                            );
                        }
                    }
                });
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui
                    .add_enabled(!busy, Button::new("Connect").min_size(Vec2::new(88.0, 28.0)))
                    .clicked()
                {
                    action = Some(DeviceAction::Connect(device.endpoint.clone()));
                }
            });
        });
    });
    if response.clicked() && !busy {
        action = Some(DeviceAction::Connect(device.endpoint.clone()));
    }
    ui.separator();
    action
}
