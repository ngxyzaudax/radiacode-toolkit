use std::time::{Duration, Instant};

use radiacode_lib::config::AlarmLimits;
use radiacode_lib::{DeviceEndpoint, DiscoveredDevice};

use crate::dosimeter::DosimeterState;
use crate::model::{ConnectionState, DeviceInfo, SpectrumView};
use crate::monitor::MonitorState;

pub struct AppState {
    pub devices: Vec<DiscoveredDevice>,
    pub connecting_endpoint: Option<DeviceEndpoint>,
    pub connection: ConnectionState,
    pub device_info: Option<DeviceInfo>,
    pub spectrum: Option<SpectrumView>,
    pub spectrum_sequence: u64,
    pub monitor: MonitorState,
    pub dosimeter: DosimeterState,
    pub scanning: bool,
    pub busy: bool,
    pub status: String,
    pub last_fetch: Option<Instant>,
    pub last_monitor_fetch: Option<Instant>,
    pub spectrum_fetch_pending: bool,
    pub monitor_fetch_pending: bool,
    pub scanned_once: bool,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            devices: Vec::new(),
            connecting_endpoint: None,
            connection: ConnectionState::Disconnected,
            device_info: None,
            spectrum: None,
            spectrum_sequence: 0,
            monitor: MonitorState::new(),
            dosimeter: DosimeterState::new(),
            scanning: false,
            busy: false,
            status: "Looking for nearby detectors…".into(),
            last_fetch: None,
            last_monitor_fetch: None,
            spectrum_fetch_pending: false,
            monitor_fetch_pending: false,
            scanned_once: false,
        }
    }

    pub fn apply_alarm_limits(&mut self, limits: AlarmLimits) {
        self.monitor.apply_limits(limits);
        self.dosimeter.apply_limits(limits);
    }

    pub(super) fn clear_pending_fetches(&mut self) {
        self.spectrum_fetch_pending = false;
        self.monitor_fetch_pending = false;
    }

    pub fn clear_session(&mut self) {
        self.connection = ConnectionState::Disconnected;
        self.connecting_endpoint = None;
        self.device_info = None;
        self.spectrum = None;
        self.spectrum_sequence = 0;
        self.monitor.on_disconnect();
        self.dosimeter.on_disconnect();
        self.last_fetch = None;
        self.last_monitor_fetch = None;
        self.clear_pending_fetches();
        self.status = disconnected_status(self.scanned_once).into();
    }

    pub fn live_refresh_due(&self, enabled: bool, interval_secs: u64) -> bool {
        let connected = self.connection == ConnectionState::Connected;
        let due = self
            .last_fetch
            .map(|t| t.elapsed() >= Duration::from_secs(interval_secs.max(1)))
            .unwrap_or(true);
        enabled && connected && due && !self.spectrum_fetch_pending
    }

    pub fn try_schedule_spectrum(&mut self) -> bool {
        if self.spectrum_fetch_pending {
            return false;
        }
        self.spectrum_fetch_pending = true;
        true
    }

    pub fn try_schedule_monitor(&mut self) -> bool {
        if self.monitor_fetch_pending {
            return false;
        }
        self.monitor_fetch_pending = true;
        true
    }
}

pub(super) fn disconnected_status(scanned_once: bool) -> &'static str {
    if scanned_once {
        "Disconnected. Choose a device to reconnect."
    } else {
        "Disconnected."
    }
}

pub(super) fn scan_status(device_count: usize) -> String {
    if device_count == 0 {
        "No RadiaCode devices found.".into()
    } else {
        format!("{device_count} device(s) available.")
    }
}
