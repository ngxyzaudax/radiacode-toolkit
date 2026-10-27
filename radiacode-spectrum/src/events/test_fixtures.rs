use std::time::Duration;

use radiacode_lib::data_buf::DeviceTicks;
use radiacode_lib::{
    AccumulatedDose, CountDisplayUnit, DeviceEndpoint, DiscoveredDevice, DoseDisplayUnit,
    MonitorPollSample, RssiStatus, TimedRates, TransportKind,
};

use super::AppState;
use crate::model::{DeviceInfo, SpectrumView};
use crate::persist::json_store::use_test_data_dir;

pub fn isolated_data_dir() -> tempfile::TempDir {
    let temp = tempfile::tempdir().expect("tempdir");
    use_test_data_dir(temp.path().to_path_buf());
    temp
}

pub fn isolated_state() -> (AppState, tempfile::TempDir) {
    let temp = isolated_data_dir();
    (AppState::new(), temp)
}

pub fn device_info() -> DeviceInfo {
    DeviceInfo {
        model: "RC-103".into(),
        serial: "RC-103-000001".into(),
        firmware: "4.12".into(),
        transport: TransportKind::Usb,
        address: "RC-103-000001".into(),
        energy_calib: [0.0, 3.0, 0.0],
        battery_percent: Some(90.0),
        temperature_c: None,
        rssi: RssiStatus::NotReported,
    }
}

pub fn discovered(serial: &str) -> DiscoveredDevice {
    DiscoveredDevice {
        endpoint: DeviceEndpoint::Usb {
            serial: serial.into(),
        },
        label: serial.into(),
        serial: Some(serial.into()),
        model: None,
        rssi: None,
    }
}

pub fn spectrum_view() -> SpectrumView {
    SpectrumView {
        duration: Duration::from_secs(10),
        a0: 0.0,
        a1: 3.0,
        a2: 0.0,
        counts: vec![1, 2, 3],
        total_counts: 6,
    }
}

pub fn monitor_sample(count_rate: f32) -> MonitorPollSample {
    MonitorPollSample {
        rates: vec![TimedRates {
            device_ts: DeviceTicks::new(100),
            dose_rate: 0.1,
            count_rate,
            dose_rate_err_pct: 5.0,
            count_rate_err_pct: 5.0,
            dose_unit: DoseDisplayUnit::MicroSievertPerHour,
            count_unit: CountDisplayUnit::Cps,
        }],
        accumulated: Some(AccumulatedDose {
            dose: 12.5,
            duration_secs: 3600,
            dose_unit: DoseDisplayUnit::MicroSievertPerHour,
        }),
        decode_warnings: 0,
        rejected_records: 0,
        resync_count: 0,
        seq_gaps: Vec::new(),
    }
}
