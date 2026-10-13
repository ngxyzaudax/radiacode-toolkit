use crate::ble_error::BleError;

const MAC_OCTETS: usize = 6;

pub fn normalize_mac(mac: &str) -> std::result::Result<String, BleError> {
    let cleaned = mac.trim().to_lowercase().replace('-', ":");
    let parts: Vec<&str> = cleaned.split(':').collect();
    if parts.len() != MAC_OCTETS || !parts.iter().all(|part| is_hex_octet(part)) {
        return Err(BleError::InvalidAddress(mac.to_string()));
    }
    Ok(cleaned)
}

fn is_hex_octet(part: &str) -> bool {
    part.len() == 2 && part.chars().all(|c| c.is_ascii_hexdigit())
}

pub fn address_matches(address: &str, normalized_target: &str) -> bool {
    address.to_lowercase() == normalized_target
}

pub fn bluez_device_path(mac: &str) -> String {
    format!(
        "/org/bluez/hci0/dev_{}",
        mac.to_uppercase().replace(':', "_")
    )
}

pub fn parse_dbus_variant_int16(stdout: &[u8]) -> Option<i16> {
    let text = String::from_utf8_lossy(stdout);
    let mut lines = text.lines().map(str::trim);
    let variant_line = lines.find(|line| line.starts_with("variant"))?;
    std::iter::once(variant_line.trim_start_matches("variant").trim_start())
        .chain(lines)
        .find_map(|line| line.strip_prefix("int16 "))
        .and_then(|value| value.trim().parse().ok())
}
