use egui::{RichText, Ui};

use radiacode_core::{DeviceEndpoint, DiscoveredDevice};

use crate::model::{ConnectionState, DeviceInfo};
use crate::theme::{ACCENT, MUTED};

use super::ui_connected::draw_connected;
use super::ui_connecting::draw_connecting;
use super::ui_discovery::draw_discovery;

pub struct DeviceViewProps<'a> {
    pub devices: &'a [DiscoveredDevice],
    pub connection: ConnectionState,
    pub connecting_endpoint: Option<&'a DeviceEndpoint>,
    pub device_info: Option<&'a DeviceInfo>,
    pub scanning: bool,
    pub busy: bool,
    pub scanned_once: bool,
    pub status: &'a str,
}

pub enum DeviceAction {
    Scan,
    Connect(DeviceEndpoint),
    Disconnect,
}

pub fn draw_device_view(ui: &mut Ui, props: DeviceViewProps<'_>) -> Option<DeviceAction> {
    ui.add_space(12.0);
    ui.label(RichText::new("Radiacode").size(28.0).color(ACCENT).strong());
    ui.add_space(4.0);
    ui.label(
        RichText::new("Connect over USB or Bluetooth and inspect device status.")
            .small()
            .color(MUTED),
    );
    ui.add_space(20.0);
    ui.set_max_width(640.0);
    let action = match props.connection {
        ConnectionState::Connected => props
            .device_info
            .map(|info| draw_connected(ui, info))
            .flatten(),
        ConnectionState::Connecting => {
            draw_connecting(ui, props.connecting_endpoint);
            None
        }
        ConnectionState::Disconnected => {
            let list_height = ui.available_height().max(200.0) - 80.0;
            draw_discovery(ui, &props, list_height.max(200.0))
        }
    };
    ui.add_space(16.0);
    if !props.status.is_empty() {
        ui.separator();
        ui.add_space(8.0);
        ui.label(RichText::new(props.status).small().color(MUTED));
    }
    action
}
