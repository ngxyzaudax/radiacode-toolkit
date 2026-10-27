use radiacode_lib::{
    DeviceEndpoint, DeviceMetadata, DeviceStatus, RssiStatus, Spectrum, TransportKind,
};

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub model: String,
    pub serial: String,
    pub firmware: String,
    pub transport: TransportKind,
    pub address: String,
    pub energy_calib: [f32; 3],
    pub battery_percent: Option<f32>,
    pub temperature_c: Option<f32>,
    pub rssi: RssiStatus,
}

impl DeviceInfo {
    pub fn from_metadata(
        metadata: DeviceMetadata,
        endpoint: &DeviceEndpoint,
        status: DeviceStatus,
    ) -> Self {
        let firmware = metadata.firmware_label();
        Self {
            model: metadata.model,
            serial: metadata.serial,
            firmware,
            transport: endpoint.transport(),
            address: endpoint.address_label().to_string(),
            energy_calib: metadata.energy_calibration.coefficients(),
            battery_percent: status.battery_percent,
            temperature_c: status.temperature_c,
            rssi: status.rssi,
        }
    }

    pub fn apply_status(&mut self, status: DeviceStatus) {
        let merged = DeviceStatus {
            battery_percent: self.battery_percent,
            temperature_c: self.temperature_c,
            rssi: self.rssi,
        }
        .merged(status);
        self.battery_percent = merged.battery_percent;
        self.temperature_c = merged.temperature_c;
        self.rssi = merged.rssi;
    }

    pub fn endpoint(&self) -> DeviceEndpoint {
        match self.transport {
            TransportKind::Bluetooth => DeviceEndpoint::Bluetooth {
                address: self.address.clone(),
            },
            TransportKind::Usb => DeviceEndpoint::Usb {
                serial: self.address.clone(),
            },
        }
    }
}

impl From<&DeviceInfo> for DeviceStatus {
    fn from(info: &DeviceInfo) -> Self {
        Self {
            battery_percent: info.battery_percent,
            temperature_c: info.temperature_c,
            rssi: info.rssi,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SpectrumView {
    pub duration: std::time::Duration,
    pub a0: f32,
    pub a1: f32,
    pub a2: f32,
    pub counts: Vec<u32>,
    pub total_counts: u64,
}

impl SpectrumView {
    pub fn from_spectrum(spectrum: Spectrum) -> Self {
        let total_counts = spectrum.total_counts();
        Self {
            duration: spectrum.duration,
            a0: spectrum.calibration.a0,
            a1: spectrum.calibration.a1,
            a2: spectrum.calibration.a2,
            counts: spectrum.counts,
            total_counts,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConnectionState {
    #[default]
    Disconnected,
    Connecting,
    Connected,
}
