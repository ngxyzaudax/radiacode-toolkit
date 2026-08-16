use egui::Ui;

use radiacode_core::{DeviceEndpoint, DiscoveredDevice};

use crate::layout::page_scroll;
use crate::model::{ConnectionState, DeviceInfo};

use super::ui_common::{draw_status_footer, COLUMN_MAX_WIDTH};
use super::ui_connected::draw_connected;
use super::ui_connecting::draw_connecting;
use super::ui_discovery::draw_discovery;

pub struct DeviceViewProps<'a> {
    pub devices: &'a [DiscoveredDevice],
    pub connection: ConnectionState,
    pub connecting_endpoint: Option<&'a DeviceEndpoint>,
    pub device_info: Option<&'a DeviceInfo>,
    pub remembered_endpoint: Option<&'a DeviceEndpoint>,
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
    let mut action = None;
    page_scroll(ui, "device_page", |ui| {
        ui.vertical_centered(|ui| {
            ui.set_max_width(COLUMN_MAX_WIDTH.min(ui.available_width()));
            action = draw_device_body(ui, props);
        });
    });
    action
}

fn draw_device_body(ui: &mut Ui, props: DeviceViewProps<'_>) -> Option<DeviceAction> {
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
            let list_height = (ui.available_height() - 160.0).max(200.0);
            draw_discovery(ui, &props, list_height)
        }
    };
    draw_status_footer(ui, props.status);
    action
}
