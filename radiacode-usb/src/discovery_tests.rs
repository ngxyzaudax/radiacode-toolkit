use radiacode_core::{DeviceEndpoint, DiscoveredDevice, TransportKind};

fn usb_device(serial: &str, label: &str) -> DiscoveredDevice {
    DiscoveredDevice {
        endpoint: DeviceEndpoint::Usb {
            serial: serial.to_string(),
        },
        label: label.to_string(),
        serial: Some(serial.to_string()),
        model: None,
        rssi: None,
    }
}

#[test]
fn discovered_device_display_label_prefers_label() {
    let device = usb_device("RC-001", "Bench RC-103");
    assert_eq!(device.display_label(), "Bench RC-103");
}

#[test]
fn discovered_device_transport_tag_usb() {
    let device = usb_device("RC-001", "RC-001");
    assert_eq!(device.transport_tag(), "USB");
    assert_eq!(device.endpoint.transport(), TransportKind::Usb);
}
