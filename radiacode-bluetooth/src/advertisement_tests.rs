use radiacode_core::DeviceEndpoint;

use crate::advertisement::{
    Advertisement, discovered_device, is_radiacode_advertisement, sort_and_dedup,
};
use crate::uuids;

fn advertisement(address: &str, local_name: Option<&str>, with_service: bool) -> Advertisement {
    Advertisement {
        address: address.to_string(),
        local_name: local_name.map(str::to_string),
        services: if with_service {
            vec![uuids::SERVICE]
        } else {
            Vec::new()
        },
        rssi: Some(-55),
    }
}

#[test]
fn matches_by_service_or_name() {
    assert!(is_radiacode_advertisement(&advertisement(
        "aa:bb:cc:dd:ee:ff",
        None,
        true
    )));
    assert!(is_radiacode_advertisement(&advertisement(
        "aa:bb:cc:dd:ee:ff",
        Some("RadiaCode#RC-103-001234"),
        false
    )));
    assert!(!is_radiacode_advertisement(&advertisement(
        "aa:bb:cc:dd:ee:ff",
        Some("Other sensor"),
        false
    )));
}

#[test]
fn discovered_device_derives_model_serial_and_label() {
    let device = discovered_device(advertisement(
        "aa:bb:cc:dd:ee:ff",
        Some("RadiaCode#RC-103-001234"),
        true,
    ));
    assert_eq!(device.serial.as_deref(), Some("RC-103-001234"));
    assert_eq!(device.model.as_deref(), Some("RC-103"));
    assert_eq!(device.label, "RC-103");
    assert_eq!(device.rssi, Some(-55));
    assert_eq!(
        device.endpoint,
        DeviceEndpoint::Bluetooth {
            address: "aa:bb:cc:dd:ee:ff".into()
        }
    );
}

#[test]
fn discovered_device_falls_back_to_name_then_default_label() {
    let named = discovered_device(advertisement("a", Some("RadiaCode"), true));
    assert_eq!(named.label, "RadiaCode");
    assert!(named.serial.is_none());
    let anonymous = discovered_device(advertisement("b", None, true));
    assert_eq!(anonymous.label, "RadiaCode");
}

#[test]
fn sort_and_dedup_orders_by_address_and_removes_duplicates() {
    let devices = vec![
        discovered_device(advertisement("cc:00:00:00:00:00", None, true)),
        discovered_device(advertisement("aa:00:00:00:00:00", None, true)),
        discovered_device(advertisement("cc:00:00:00:00:00", None, true)),
    ];
    let sorted = sort_and_dedup(devices);
    let addresses: Vec<&str> = sorted
        .iter()
        .map(|device| device.endpoint.address_label())
        .collect();
    assert_eq!(addresses, vec!["aa:00:00:00:00:00", "cc:00:00:00:00:00"]);
}
