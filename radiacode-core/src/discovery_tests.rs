use radiacode_protocol::{CountDisplayUnit, DoseDisplayUnit};

use crate::discovery::{
    DeviceEndpoint, DiscoveredDevice, TransportKind, merge_discovered, resolve_usb_endpoint,
};
use crate::rate_units::{count_unit_label, dose_accum_unit_label, dose_unit_label};

fn usb(serial: &str) -> DiscoveredDevice {
    DiscoveredDevice {
        endpoint: DeviceEndpoint::Usb {
            serial: serial.into(),
        },
        label: String::new(),
        serial: Some(serial.into()),
        model: None,
        rssi: None,
    }
}

fn bluetooth(address: &str, serial: Option<&str>) -> DiscoveredDevice {
    DiscoveredDevice {
        endpoint: DeviceEndpoint::Bluetooth {
            address: address.into(),
        },
        label: String::new(),
        serial: serial.map(Into::into),
        model: Some("RC-103".into()),
        rssi: Some(-50),
    }
}

#[test]
fn endpoint_transport_and_address_label() {
    let usb_endpoint = usb("RC-103-1").endpoint;
    assert_eq!(usb_endpoint.transport(), TransportKind::Usb);
    assert_eq!(usb_endpoint.address_label(), "RC-103-1");
    let bluetooth_endpoint = bluetooth("AA:BB", None).endpoint;
    assert_eq!(bluetooth_endpoint.transport(), TransportKind::Bluetooth);
    assert_eq!(bluetooth_endpoint.address_label(), "AA:BB");
    assert_eq!(TransportKind::Usb.label(), "USB");
    assert_eq!(TransportKind::Bluetooth.label(), "Bluetooth");
}

#[test]
fn display_label_prefers_explicit_label_then_model_then_serial() {
    let mut device = bluetooth("AA:BB", Some("RC-103-9"));
    assert_eq!(device.display_label(), "RC-103");
    device.model = None;
    assert_eq!(device.display_label(), "RC-103-9");
    device.serial = None;
    assert_eq!(device.display_label(), "RadiaCode");
    device.label = "Named".into();
    assert_eq!(device.display_label(), "Named");
}

#[test]
fn merge_discovered_drops_bluetooth_duplicates_by_serial() {
    let merged = merge_discovered(
        vec![usb("RC-103-1")],
        vec![
            bluetooth("AA", Some("RC-103-1")),
            bluetooth("BB", Some("RC-103-2")),
            bluetooth("CC", None),
        ],
    );
    assert_eq!(merged.len(), 3);
    assert_eq!(merged[1].endpoint.address_label(), "BB");
    assert_eq!(merged[2].endpoint.address_label(), "CC");
}

#[test]
fn resolve_usb_endpoint_rules() {
    let preferred = DeviceEndpoint::Usb {
        serial: "RC-103-2".into(),
    };
    assert_eq!(resolve_usb_endpoint(&[], &preferred), preferred);
    assert_eq!(
        resolve_usb_endpoint(&[bluetooth("AA", None)], &preferred),
        preferred
    );
    assert_eq!(
        resolve_usb_endpoint(&[usb("RC-103-7")], &preferred),
        usb("RC-103-7").endpoint
    );
    let many = [usb("RC-103-1"), usb("RC-103-2")];
    assert_eq!(resolve_usb_endpoint(&many, &preferred), preferred);
    let by_serial = [
        usb("RC-103-1"),
        DiscoveredDevice {
            endpoint: DeviceEndpoint::Usb {
                serial: "bus-2".into(),
            },
            ..usb("RC-103-2")
        },
    ];
    assert_eq!(
        resolve_usb_endpoint(&by_serial, &preferred).address_label(),
        "bus-2"
    );
    let unmatched = [usb("RC-103-8"), usb("RC-103-9")];
    assert_eq!(
        resolve_usb_endpoint(&unmatched, &preferred).address_label(),
        "RC-103-8"
    );
}

#[test]
fn unit_labels() {
    assert_eq!(
        dose_unit_label(DoseDisplayUnit::MicroSievertPerHour),
        "µSv/h"
    );
    assert_eq!(
        dose_unit_label(DoseDisplayUnit::MicroRoentgenPerHour),
        "µR/h"
    );
    assert_eq!(
        dose_accum_unit_label(DoseDisplayUnit::MicroSievertPerHour),
        "µSv"
    );
    assert_eq!(
        dose_accum_unit_label(DoseDisplayUnit::MicroRoentgenPerHour),
        "µR"
    );
    assert_eq!(count_unit_label(CountDisplayUnit::Cpm), "cpm");
    assert_eq!(count_unit_label(CountDisplayUnit::Cps), "cps");
}
