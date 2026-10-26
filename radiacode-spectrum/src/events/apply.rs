use std::time::Instant;

use radiacode_lib::{DeviceEndpoint, DeviceStatus, DiscoveredDevice, MonitorPollSample};
use tracing::{debug, info, warn};

use super::state::{AppState, scan_status};
use crate::model::{ConnectionState, DeviceInfo, SpectrumView};
use crate::worker::{WorkerCommand, WorkerEvent};

impl AppState {
    pub fn apply_event(
        &mut self,
        event: WorkerEvent,
        accept_session: bool,
    ) -> Option<WorkerCommand> {
        match event {
            WorkerEvent::Busy(busy) => {
                debug!(busy, "worker busy state");
                self.busy = busy;
                None
            }
            WorkerEvent::ScanFinished(devices) => self.on_scan_finished(devices),
            WorkerEvent::Disconnected => {
                info!("ui disconnected");
                self.clear_session();
                None
            }
            WorkerEvent::Error(message) => self.on_error(message),
            WorkerEvent::DeviceConfig(_) => None,
            session_event if !accept_session => {
                debug!(event = ?session_event, "ignoring event from ended session");
                None
            }
            WorkerEvent::Connected(info) => self.on_connected(info),
            WorkerEvent::Reconnecting => self.on_reconnecting(),
            WorkerEvent::UsbPermissionRequired { endpoint } => {
                self.on_usb_permission_required(endpoint)
            }
            WorkerEvent::Spectrum(spectrum) => self.on_spectrum(spectrum),
            WorkerEvent::DeviceStatus(status) => self.on_device_status(status),
            WorkerEvent::MonitorSample(sample) => self.on_monitor_sample(sample),
            WorkerEvent::DoseResetComplete => {
                self.dosimeter.on_reset();
                self.status = "Dose reset.".into();
                Some(WorkerCommand::FetchMonitor)
            }
            WorkerEvent::MonitorPollComplete => {
                self.last_monitor_fetch = Some(Instant::now());
                self.monitor_fetch_pending = false;
                None
            }
            WorkerEvent::AlarmLimits(limits) => {
                self.apply_alarm_limits(limits);
                None
            }
        }
    }

    fn on_scan_finished(&mut self, devices: Vec<DiscoveredDevice>) -> Option<WorkerCommand> {
        info!(count = devices.len(), "ui received scan results");
        self.scanning = false;
        self.scanned_once = true;
        self.status = scan_status(devices.len());
        self.devices = devices;
        None
    }

    fn on_connected(&mut self, info: DeviceInfo) -> Option<WorkerCommand> {
        info!(serial = %info.serial, address = %info.address, transport = ?info.transport, "ui connected");
        let fresh_session = self.connection != ConnectionState::Connected;
        self.connection = ConnectionState::Connected;
        self.connecting_endpoint = None;
        if fresh_session {
            self.monitor.on_connect();
            self.dosimeter.on_connect(&info.serial);
        }
        self.device_info = Some(info);
        self.clear_pending_fetches();
        self.status = "Connected. Acquiring monitor data…".into();
        Some(WorkerCommand::FetchMonitor)
    }

    fn on_reconnecting(&mut self) -> Option<WorkerCommand> {
        warn!("ui reconnecting after connection loss");
        self.status = "Connection lost, reconnecting…".into();
        self.spectrum = None;
        self.last_fetch = None;
        self.monitor.on_reconnecting();
        self.clear_pending_fetches();
        None
    }

    fn on_usb_permission_required(&mut self, endpoint: DeviceEndpoint) -> Option<WorkerCommand> {
        warn!(?endpoint, "ui usb permission required");
        self.scanning = false;
        self.clear_pending_fetches();
        self.connection = ConnectionState::Disconnected;
        self.connecting_endpoint = Some(endpoint);
        self.status = "USB access required.".into();
        None
    }

    fn on_spectrum(&mut self, spectrum: SpectrumView) -> Option<WorkerCommand> {
        self.spectrum = Some(spectrum);
        self.spectrum_sequence = self.spectrum_sequence.saturating_add(1);
        self.last_fetch = Some(Instant::now());
        self.spectrum_fetch_pending = false;
        self.status = "Live spectrum".into();
        debug!(sequence = self.spectrum_sequence, "ui spectrum updated");
        None
    }

    fn on_device_status(&mut self, status: DeviceStatus) -> Option<WorkerCommand> {
        self.monitor_fetch_pending = false;
        if let Some(info) = self.device_info.as_mut() {
            info.apply_status(status);
            debug!(battery = ?info.battery_percent, temperature = ?info.temperature_c, rssi = ?info.rssi_dbm, "ui device status updated");
        }
        None
    }

    fn on_monitor_sample(&mut self, sample: MonitorPollSample) -> Option<WorkerCommand> {
        self.monitor.push_poll(
            &sample.rates,
            sample.decode_warnings,
            sample.rejected_records,
            sample.resync_count,
            &sample.seq_gaps,
        );
        if let Some(accumulated) = sample.accumulated {
            self.dosimeter.push_sample(accumulated);
        }
        self.last_monitor_fetch = Some(Instant::now());
        self.monitor_fetch_pending = false;
        self.status = "Live monitor".into();
        None
    }

    fn on_error(&mut self, message: String) -> Option<WorkerCommand> {
        warn!(%message, "ui received worker error");
        self.scanning = false;
        self.clear_pending_fetches();
        if self.connection == ConnectionState::Connecting {
            self.connection = ConnectionState::Disconnected;
            self.connecting_endpoint = None;
        }
        self.status = message;
        None
    }
}
