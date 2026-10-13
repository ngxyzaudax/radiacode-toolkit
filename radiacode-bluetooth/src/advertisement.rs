use radiacode_core::{
    DeviceEndpoint, DiscoveredDevice, model_from_advertisement, serial_from_advertisement,
};
use uuid::Uuid;

use crate::uuids;

const DEFAULT_LABEL: &str = "RadiaCode";

pub struct Advertisement {
    pub address: String,
    pub local_name: Option<String>,
    pub services: Vec<Uuid>,
    pub rssi: Option<i16>,
}

pub fn is_radiacode_advertisement(advertisement: &Advertisement) -> bool {
    let advertises_service = advertisement.services.contains(&uuids::SERVICE);
    let name_matches = advertisement
        .local_name
        .as_deref()
        .is_some_and(|name| name.to_ascii_lowercase().contains("radiacode"));
    advertises_service || name_matches
}

pub fn discovered_device(advertisement: Advertisement) -> DiscoveredDevice {
    let local_name = advertisement.local_name;
    let serial = local_name.as_deref().and_then(serial_from_advertisement);
    let model = local_name.as_deref().and_then(model_from_advertisement);
    DiscoveredDevice {
        endpoint: DeviceEndpoint::Bluetooth {
            address: advertisement.address,
        },
        label: device_label(model.as_deref(), serial.as_deref(), local_name.as_deref()),
        serial,
        model,
        rssi: advertisement.rssi,
    }
}

fn device_label(model: Option<&str>, serial: Option<&str>, local_name: Option<&str>) -> String {
    model
        .or(serial)
        .or(local_name)
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
