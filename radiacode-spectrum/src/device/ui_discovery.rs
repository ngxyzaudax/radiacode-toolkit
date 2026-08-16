use egui::{Button, RichText, Ui};

use radiacode_core::{DeviceEndpoint, DiscoveredDevice, TransportKind};

use crate::theme::{MUTED, SPACE_MD};

use super::ui_common::draw_section_heading;
use super::ui_device_row::{draw_device_row, draw_reconnect_card};
use super::ui_empty::draw_empty_discovery;
use super::DeviceAction;
use super::DeviceViewProps;

pub fn draw_discovery(
    ui: &mut Ui,
    props: &DeviceViewProps<'_>,
    max_list_height: f32,
) -> Option<DeviceAction> {
    let mut action = None;
    if let Some(next) = draw_discovery_header(ui, props) {
        action = Some(next);
    }
    ui.add_space(SPACE_MD);
    if let Some(next) = draw_discovery_body(ui, props, max_list_height) {
        action = Some(next);
    }
    action
}

fn draw_discovery_header(ui: &mut Ui, props: &DeviceViewProps<'_>) -> Option<DeviceAction> {
    let mut action = None;
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(RichText::new("Connect a detector").size(20.0).strong());
            ui.label(
                RichText::new("USB or Bluetooth RadiaCode devices on this computer.")
                    .small()
                    .color(MUTED),
            );
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if props.scanning {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(RichText::new("Scanning…").small().color(MUTED));
                });
            } else if ui
                .add_enabled(!props.busy, Button::new("Scan again"))
                .clicked()
            {
                action = Some(DeviceAction::Scan);
            }
        });
    });
    action
}

fn draw_discovery_body(
    ui: &mut Ui,
    props: &DeviceViewProps<'_>,
    max_list_height: f32,
) -> Option<DeviceAction> {
    let mut action = None;
    if let Some(endpoint) = props.remembered_endpoint {
        draw_section_heading(ui, "Last used");
        let device = props
            .devices
            .iter()
            .find(|entry| &entry.endpoint == endpoint);
        if let Some(next) = draw_reconnect_card(ui, endpoint, device, props.busy) {
            action = Some(next);
        }
        ui.add_space(SPACE_MD);
    }
    let available = available_devices(props.devices, props.remembered_endpoint);
    if available.is_empty() && !props.scanning {
        if let Some(next) = draw_empty_discovery(ui, props.scanned_once, !props.busy) {
            action = Some(next);
        }
        return action;
    }
    if !available.is_empty() {
        draw_section_heading(ui, &format!("Available devices ({})", available.len()));
        egui::ScrollArea::vertical()
            .max_height(max_list_height)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for device in available {
                    if let Some(next) = draw_device_row(ui, device, props.busy) {
                        action = Some(next);
                    }
                }
            });
    }
    action
}

fn available_devices<'a>(
    devices: &'a [DiscoveredDevice],
    remembered: Option<&DeviceEndpoint>,
) -> Vec<&'a DiscoveredDevice> {
    let mut usb: Vec<_> = devices
        .iter()
        .filter(|device| device.endpoint.transport() == TransportKind::Usb)
        .filter(|device| !remembered.is_some_and(|endpoint| endpoint == &device.endpoint))
        .collect();
    let mut bluetooth: Vec<_> = devices
        .iter()
        .filter(|device| device.endpoint.transport() == TransportKind::Bluetooth)
        .filter(|device| !remembered.is_some_and(|endpoint| endpoint == &device.endpoint))
        .collect();
    bluetooth.sort_by_key(|device| std::cmp::Reverse(device.rssi));
    usb.extend(bluetooth);
    usb
}
