use radiacode_core::DeviceEndpoint;

use crate::transport::{UsbIdentity, discovered_device, select_device_index, sort_and_dedup};

fn identity(address: u8, serial: Option<&str>, product: Option<&str>) -> UsbIdentity {
    UsbIdentity {
        address,
        serial: serial.map(str::to_string),
        product: product.map(str::to_string),
    }
}

#[test]
fn serial_target_selects_matching_device() {
    let addresses = vec!["3".to_string(), "7".to_string()];
    let serials = vec![
        Some("RC-103-000001".to_string()),
        Some("RC-102-000002".to_string()),
    ];
    assert_eq!(
        select_device_index("RC-102-000002", &addresses, &serials),
        Some(1)
    );
    assert_eq!(select_device_index("usb-3", &addresses, &serials), Some(0));
    assert_eq!(select_device_index("RC-999", &addresses, &serials), None);
}

#[test]
fn single_device_is_selected_when_target_is_unknown() {
    let addresses = vec!["3".to_string()];
    let serials = vec![None];
    assert_eq!(select_device_index("RC-999", &addresses, &serials), Some(0));
}

#[test]
fn discovered_device_prefers_model_label_and_serial_endpoint() {
    let device = discovered_device(identity(4, Some("RC-103-000123"), Some("RadiaCode")));
    assert_eq!(device.label, "RC-103");
    assert_eq!(device.model.as_deref(), Some("RC-103"));
    assert_eq!(
        device.endpoint,
        DeviceEndpoint::Usb {
            serial: "RC-103-000123".into()
        }
    );
}

#[test]
fn discovered_device_without_serial_uses_address_endpoint_and_product_label() {
    let device = discovered_device(identity(9, Some(""), Some("RadiaCode 10x")));
    assert_eq!(device.label, "RadiaCode 10x");
    assert!(device.serial.is_none());
    assert_eq!(device.endpoint.address_label(), "usb-9");
    let anonymous = discovered_device(identity(2, None, None));
    assert_eq!(anonymous.label, "RadiaCode");
}

#[test]
fn sort_and_dedup_orders_by_endpoint() {
    let devices = vec![
        discovered_device(identity(2, Some("RC-B"), None)),
        discovered_device(identity(1, Some("RC-A"), None)),
        discovered_device(identity(3, Some("RC-A"), None)),
    ];
    let sorted = sort_and_dedup(devices);
    let labels: Vec<&str> = sorted
        .iter()
        .map(|device| device.endpoint.address_label())
        .collect();
    assert_eq!(labels, vec!["RC-A", "RC-B"]);
}
