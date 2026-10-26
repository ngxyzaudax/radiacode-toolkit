use radiacode_lib::discovery::{merge_discovered, resolve_usb_endpoint};
use radiacode_lib::{DeviceEndpoint, TransportKind};
use tracing::{info, warn};

use super::SpectrumApp;
use crate::model::{ConnectionState, DeviceInfo};
use crate::monitor::AlarmLevel;
use crate::usb_access::{UsbAccessPrompt, usb_access_required};
use crate::worker::WorkerCommand;

impl SpectrumApp {
    pub(super) fn remember_connected_device(&mut self, info: &DeviceInfo) {
        if !self.settings.app.remember_device {
            return;
        }
        self.settings.app.last_endpoint = Some(info.endpoint());
        self.settings.persist_app();
    }

    fn auto_connect_target(&self) -> Option<DeviceEndpoint> {
        if self.shutting_down()
            || self.auto_connect_attempted
            || !self.settings.app.auto_connect
            || self.state.connection != ConnectionState::Disconnected
        {
            return None;
        }
        let endpoint = self.settings.app.last_endpoint.clone()?;
        self.state
            .devices
            .iter()
            .any(|device| device.endpoint == endpoint)
            .then_some(endpoint)
    }

    pub(super) fn maybe_auto_connect(&mut self) {
        let Some(endpoint) = self.auto_connect_target() else {
            return;
        };
        if usb_access_required(&endpoint).is_some() {
            info!(?endpoint, "auto-connect skipped: usb access required");
            return;
        }
        self.auto_connect_attempted = true;
        info!(?endpoint, "auto-connecting to remembered device");
        self.start_connect(endpoint);
    }

    pub(super) fn start_scan(&mut self) {
        info!("ui starting scan");
        self.state.scanning = true;
        self.state.status = "Scanning for RadiaCode devices…".into();
        self.send(WorkerCommand::Scan);
    }

    pub(super) fn refresh_usb_devices(&mut self) {
        let bluetooth: Vec<_> = self
            .state
            .devices
            .iter()
            .filter(|device| device.endpoint.transport() == TransportKind::Bluetooth)
            .cloned()
            .collect();
        match crate::usb_access::scan_usb_devices() {
            Ok(usb) => self.state.devices = merge_discovered(usb, bluetooth),
            Err(error) => warn!(%error, "usb rescan failed"),
        }
    }

    pub(super) fn start_connect(&mut self, endpoint: DeviceEndpoint) {
        self.start_connect_internal(endpoint, false);
    }

    pub(super) fn connect_resolved(&mut self, preferred: &DeviceEndpoint) {
        self.refresh_usb_devices();
        let endpoint = resolve_usb_endpoint(&self.state.devices, preferred);
        self.start_connect_internal(endpoint, true);
    }

    pub(super) fn start_connect_internal(&mut self, endpoint: DeviceEndpoint, force_usb: bool) {
        let endpoint = resolve_usb_endpoint(&self.state.devices, &endpoint);
        if !force_usb && let Some(status) = usb_access_required(&endpoint) {
            info!(?endpoint, ?status, "usb access required before connect");
            self.session_blocked = false;
            self.state.connection = ConnectionState::Disconnected;
            self.state.connecting_endpoint = Some(endpoint.clone());
            self.state.status = "USB access required.".into();
            self.usb_access_prompt = Some(UsbAccessPrompt::new(endpoint, status));
            return;
        }
        let address = endpoint.address_label().to_string();
        info!(%address, ?endpoint, force_usb, "ui starting connect");
        self.session_blocked = false;
        self.state.connection = ConnectionState::Connecting;
        self.state.connecting_endpoint = Some(endpoint.clone());
        self.state.status = format!("Connecting to {address}…");
        let hint_rssi = self.hint_rssi_for_endpoint(&endpoint);
        self.send(WorkerCommand::Connect {
            endpoint,
            hint_rssi,
        });
    }

    fn hint_rssi_for_endpoint(&self, endpoint: &DeviceEndpoint) -> Option<i16> {
        self.state
            .devices
            .iter()
            .find(|device| device.endpoint == *endpoint)
            .and_then(|device| device.rssi)
    }

    pub(super) fn disconnect_device(&mut self) {
        info!("ui disconnect requested");
        self.worker.end_session();
        self.session_blocked = true;
        self.state.clear_session();
        self.spectrogram.on_disconnect();
        self.settings.on_disconnect();
        self.last_alarm_level = AlarmLevel::Normal;
        self.send(WorkerCommand::Disconnect);
    }
}
