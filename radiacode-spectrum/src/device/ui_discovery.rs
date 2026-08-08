use egui::{RichText, Ui};

use radiacode_core::TransportKind;

use crate::theme::MUTED;

use super::DeviceAction;
use super::DeviceViewProps;

pub fn draw_discovery(ui: &mut Ui, props: &DeviceViewProps<'_>, max_list_height: f32) -> Option<DeviceAction> {
    let mut action = None;
    ui.horizontal(|ui| {
        ui.label(RichText::new("Nearby devices").size(18.0).strong());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let label = if props.scanning { "Scanning…" } else { "Scan" };
            if ui
                .add_enabled(!props.busy && !props.scanning, egui::Button::new(label))
                .clicked()
            {
                action = Some(DeviceAction::Scan);
            }
        });
    });
    ui.add_space(12.0);

    if props.scanning {
        ui.horizontal(|ui| {
            ui.spinner();
            ui.label("Searching for RadiaCode over USB and Bluetooth…");
        });
        return action;
    }

    if props.devices.is_empty() {
        draw_empty_discovery(ui, props.scanned_once);
        return action;
    }

    egui::ScrollArea::vertical()
        .max_height(max_list_height)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for device in props.devices {
                ui.add_space(6.0);
                egui::Frame::group(ui.style()).show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(device.display_label()).strong().size(16.0));
                                ui.label(
                                    RichText::new(device.transport_tag())
                                        .small()
                                        .color(MUTED),
                                );
                            });
                            if let Some(serial) = &device.serial {
                                ui.label(RichText::new(serial).small());
                            }
                            ui.label(
                                RichText::new(device.endpoint.address_label())
                                    .monospace()
                                    .small(),
                            );
                            if device.endpoint.transport() == TransportKind::Bluetooth {
                                if let Some(rssi) = device.rssi {
                                    ui.label(RichText::new(format!("{rssi} dBm")).weak().small());
                                }
                            }
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui
                                .add_enabled(!props.busy, egui::Button::new("Connect"))
                                .clicked()
                            {
                                action = Some(DeviceAction::Connect(device.endpoint.clone()));
                            }
                        });
                    });
                });
            }
        });
    action
}

fn draw_empty_discovery(ui: &mut Ui, scanned_once: bool) {
    if scanned_once {
        ui.label(RichText::new("No detectors found.").size(16.0));
        ui.label(
            RichText::new("Plug in USB, power on Bluetooth, then scan again.")
                .weak()
                .small(),
        );
    } else {
        ui.label(RichText::new("Starting USB and Bluetooth discovery…").weak());
    }
}
