use crate::address::{address_matches, bluez_device_path, normalize_mac, parse_dbus_variant_int16};
use crate::ble_error::BleError;

#[test]
fn normalize_mac_accepts_dash_and_mixed_case() {
    assert_eq!(
        normalize_mac(" 52-43-0A-bb-CC-dd ").unwrap(),
        "52:43:0a:bb:cc:dd"
    );
}

#[test]
fn normalize_mac_rejects_malformed_addresses() {
    for input in ["52:43:0a:bb:cc", "52:43:0a:bb:cc:zz", "52430abbccdd", ""] {
        assert!(
            matches!(normalize_mac(input), Err(BleError::InvalidAddress(_))),
            "{input:?} should be rejected"
        );
    }
}

#[test]
fn address_matches_ignores_case() {
    assert!(address_matches("52:43:0A:BB:CC:DD", "52:43:0a:bb:cc:dd"));
    assert!(!address_matches("52:43:0A:BB:CC:DE", "52:43:0a:bb:cc:dd"));
}

#[test]
fn bluez_device_path_uses_upper_case_underscores() {
    assert_eq!(
        bluez_device_path("52:43:0a:bb:cc:dd"),
        "/org/bluez/hci0/dev_52_43_0A_BB_CC_DD"
    );
}

#[test]
fn dbus_variant_on_single_line_is_parsed() {
    let output = b"method return time=1.0 sender=:1.3 -> destination=:1.9 serial=5 reply_serial=2\n   variant       int16 -62\n";
    assert_eq!(parse_dbus_variant_int16(output), Some(-62));
}

#[test]
fn dbus_variant_on_separate_lines_is_parsed() {
    let output = b"method return\n   variant\n      int16 -70\n";
    assert_eq!(parse_dbus_variant_int16(output), Some(-70));
}

#[test]
fn dbus_output_without_variant_yields_none() {
    assert_eq!(
        parse_dbus_variant_int16(b"Error org.freedesktop.DBus.Error"),
        None
    );
    assert_eq!(parse_dbus_variant_int16(b"variant string \"x\""), None);
}
