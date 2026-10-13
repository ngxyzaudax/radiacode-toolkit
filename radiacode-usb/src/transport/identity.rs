use radiacode_core::{DeviceEndpoint, DiscoveredDevice, model_from_serial};

const DEFAULT_LABEL: &str = "RadiaCode";
const ADDRESS_SERIAL_PREFIX: &str = "usb-";

pub struct UsbIdentity {
    pub address: u8,
    pub serial: Option<String>,
    pub product: Option<String>,
}

pub fn select_device_index(
    target: &str,
    addresses: &[String],
    serials: &[Option<String>],
) -> Option<usize> {
    let matched = match target.strip_prefix(ADDRESS_SERIAL_PREFIX) {
        Some(address) => addresses.iter().position(|value| value == address),
        None => serials
            .iter()
            .position(|value| value.as_deref() == Some(target)),
    };
    matched.or((addresses.len() == 1).then_some(0))
}

pub fn discovered_device(identity: UsbIdentity) -> DiscoveredDevice {
    let serial = non_empty(identity.serial);
    let product = non_empty(identity.product);
    let model = serial.as_deref().map(model_from_serial);
    DiscoveredDevice {
        endpoint: DeviceEndpoint::Usb {
            serial: endpoint_serial(serial.as_deref(), identity.address),
        },
        label: device_label(model.as_deref(), serial.as_deref(), product.as_deref()),
        serial,
        model,
        rssi: None,
    }
}

fn non_empty(value: Option<String>) -> Option<String> {
    value.filter(|value| !value.is_empty())
}

fn endpoint_serial(serial: Option<&str>, address: u8) -> String {
    serial
        .map(str::to_string)
        .unwrap_or_else(|| format!("{ADDRESS_SERIAL_PREFIX}{address}"))
}

fn device_label(model: Option<&str>, serial: Option<&str>, product: Option<&str>) -> String {
    model
        .or(serial)
        .or(product)
        .unwrap_or(DEFAULT_LABEL)
        .to_string()
}

pub fn sort_and_dedup(mut devices: Vec<DiscoveredDevice>) -> Vec<DiscoveredDevice> {
    devices.sort_by(|left, right| {
        left.endpoint
            .address_label()
            .cmp(right.endpoint.address_label())
    });
    devices.dedup_by(|left, right| left.endpoint == right.endpoint);
    devices
}
