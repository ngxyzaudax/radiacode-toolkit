use std::sync::{Arc, Mutex};

use crossbeam_channel::Sender;
use radiacode_core::{DeviceEndpoint, DiscoveredDevice, TransportKind};
use tokio::sync::mpsc::UnboundedReceiver;

use super::SpectrumApp;
use crate::events::test_fixtures::{device_info, isolated_data_dir};
use crate::model::{ConnectionState, DeviceInfo};
use crate::spectrogram::capture::SpectrogramCapture;
use crate::worker::{WorkerCommand, WorkerEvent, test_worker};

pub struct AppHarness {
    pub app: SpectrumApp,
    pub commands: UnboundedReceiver<WorkerCommand>,
    pub events: Sender<WorkerEvent>,
    _data_dir: tempfile::TempDir,
}

impl AppHarness {
    pub fn new() -> Self {
        let data_dir = isolated_data_dir();
        let worker = test_worker();
        let capture = Arc::new(Mutex::new(SpectrogramCapture::new()));
        Self {
            app: SpectrumApp::with_worker(worker.handle, capture),
            commands: worker.commands,
            events: worker.events,
            _data_dir: data_dir,
        }
    }

    pub fn connected() -> Self {
        let mut harness = Self::new();
        harness.app.start_connect(bluetooth_endpoint());
        harness.deliver(WorkerEvent::Connected(connected_info()));
        harness.drain_commands();
        assert_eq!(harness.app.state.connection, ConnectionState::Connected);
        harness
    }

    pub fn deliver(&mut self, event: WorkerEvent) {
        self.events.send(event).expect("event channel");
        self.app.poll_events();
    }

    pub fn drain_commands(&mut self) -> Vec<WorkerCommand> {
        let mut drained = Vec::new();
        while let Ok(command) = self.commands.try_recv() {
            drained.push(command);
        }
        drained
    }
}

pub const BLUETOOTH_ADDRESS: &str = "AA:BB:CC:DD:EE:FF";

pub fn bluetooth_endpoint() -> DeviceEndpoint {
    DeviceEndpoint::Bluetooth {
        address: BLUETOOTH_ADDRESS.into(),
    }
}

pub fn connected_info() -> DeviceInfo {
    DeviceInfo {
        transport: TransportKind::Bluetooth,
        address: BLUETOOTH_ADDRESS.into(),
        rssi_dbm: Some(-55),
        ..device_info()
    }
}

pub fn discovered_bluetooth(rssi: Option<i16>) -> DiscoveredDevice {
    DiscoveredDevice {
        endpoint: bluetooth_endpoint(),
        label: "RC-103".into(),
        serial: Some("RC-103-000001".into()),
        model: Some("RC-103".into()),
        rssi,
    }
}

pub fn has_command(commands: &[WorkerCommand], matcher: fn(&WorkerCommand) -> bool) -> bool {
    commands.iter().any(matcher)
}
