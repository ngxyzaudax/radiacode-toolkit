use std::time::Duration;

use radiacode_lib::protocol::{DeviceVersions, FirmwareVersion};
use radiacode_lib::{
    DeviceEndpoint, DeviceMetadata, DeviceStatus, EnergyCalibration, RssiStatus, Spectrum,
    TransportKind,
};

use crate::model::{DeviceInfo, SpectrumView};

fn version(major: u16, minor: u16) -> FirmwareVersion {
    FirmwareVersion {
        major,
        minor,
        date: "Jan 01 2024".into(),
    }
}

fn metadata() -> DeviceMetadata {
    DeviceMetadata {
        serial: "RC-103-000001".into(),
        model: "RC-103".into(),
        versions: DeviceVersions {
            boot: version(4, 0),
            target: version(4, 12),
        },
        energy_calibration: EnergyCalibration::new(1.0, 2.0, 3.0),
    }
}

fn status(battery: Option<f32>, temperature: Option<f32>, rssi: Option<i16>) -> DeviceStatus {
    DeviceStatus {
        battery_percent: battery,
        temperature_c: temperature,
        rssi: rssi.map(RssiStatus::Measured).unwrap_or_default(),
    }
}

#[test]
fn device_info_from_metadata_captures_endpoint_and_status() {
    let endpoint = DeviceEndpoint::Bluetooth {
        address: "AA:BB".into(),
    };
    let info =
        DeviceInfo::from_metadata(metadata(), &endpoint, status(Some(50.0), None, Some(-70)));
    assert_eq!(info.firmware, "4.12 (Jan 01 2024)");
    assert_eq!(info.transport, TransportKind::Bluetooth);
    assert_eq!(info.address, "AA:BB");
    assert_eq!(info.endpoint(), endpoint);
    assert_eq!(info.battery_percent, Some(50.0));
    assert_eq!(info.rssi, RssiStatus::Measured(-70));
    assert_eq!(
        DeviceStatus::from(&info),
        status(Some(50.0), None, Some(-70))
    );
}

#[test]
fn usb_endpoint_round_trips_through_serial() {
    let endpoint = DeviceEndpoint::Usb {
        serial: "RC-103-000001".into(),
    };
    let info = DeviceInfo::from_metadata(metadata(), &endpoint, DeviceStatus::default());
    assert_eq!(info.endpoint(), endpoint);
    assert_eq!(info.transport.label(), "USB");
}

#[test]
fn apply_status_keeps_known_values_when_update_is_partial() {
    let endpoint = DeviceEndpoint::Usb { serial: "X".into() };
    let mut info =
        DeviceInfo::from_metadata(metadata(), &endpoint, status(Some(80.0), Some(20.0), None));
    info.apply_status(status(None, Some(25.0), Some(-60)));
    assert_eq!(info.battery_percent, Some(80.0));
    assert_eq!(info.temperature_c, Some(25.0));
    assert_eq!(info.rssi, RssiStatus::Measured(-60));
}

#[test]
fn spectrum_view_sums_counts() {
    let view = SpectrumView::from_spectrum(Spectrum {
        duration: Duration::from_secs(30),
        calibration: EnergyCalibration::new(0.5, 3.0, 0.0),
        counts: vec![u32::MAX, 1, 2],
    });
    assert_eq!(view.total_counts, u64::from(u32::MAX) + 3);
    assert_eq!(view.duration, Duration::from_secs(30));
    assert_eq!(view.counts.len(), 3);
}
