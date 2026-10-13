use radiacode_core::{
    AlarmLimits, DeviceConfig, DeviceEndpoint, DeviceStatus, DiscoveredDevice, MonitorPollSample,
};

use crate::model::{DeviceInfo, SpectrumView};

#[derive(Debug, Clone)]
pub enum WorkerCommand {
    Scan,
    Connect {
        endpoint: DeviceEndpoint,
        hint_rssi: Option<i16>,
    },
    Disconnect,
    FetchSpectrum,
    ResetSpectrum,
    ResetDose,
    FetchMonitor,
    SetCaptureInterval(f64),
    SetMonitorPollInterval(u64),
    FetchDeviceConfig,
    ApplyDeviceConfig(DeviceConfig),
    SyncDeviceClock,
}

#[derive(Debug, Clone)]
pub enum WorkerEvent {
    ScanFinished(Vec<DiscoveredDevice>),
    Connected(DeviceInfo),
    Disconnected,
    Reconnecting,
    UsbPermissionRequired { endpoint: DeviceEndpoint },
    Spectrum(SpectrumView),
    DeviceStatus(DeviceStatus),
    MonitorSample(MonitorPollSample),
    DoseResetComplete,
    MonitorPollComplete,
    AlarmLimits(AlarmLimits),
    DeviceConfig(DeviceConfig),
    Error(String),
    Busy(bool),
}
