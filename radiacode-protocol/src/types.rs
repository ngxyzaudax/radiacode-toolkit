use std::time::Duration;

#[derive(Debug, Clone, PartialEq)]
pub struct FirmwareVersion {
    pub major: u16,
    pub minor: u16,
    pub date: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeviceVersions {
    pub boot: FirmwareVersion,
    pub target: FirmwareVersion,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Spectrum {
    pub duration: Duration,
    pub a0: f32,
    pub a1: f32,
    pub a2: f32,
    pub counts: Vec<u32>,
}

pub fn channel_to_energy(channel: u32, a0: f32, a1: f32, a2: f32) -> f32 {
    let x = channel as f32;
    a0 + a1 * x + a2 * x * x
}
