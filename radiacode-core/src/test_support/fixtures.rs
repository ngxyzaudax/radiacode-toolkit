use radiacode_protocol::{
    CountDisplayUnit, DoseDisplayUnit, Error as ProtocolError, VirtSfr, VirtString,
};

use super::encode::{rare_record, real_time_record, spectrum_v0_payload};
use super::virtual_device::VirtualDevice;
use crate::error::Error;
use crate::types::AlarmLimits;

pub fn protocol_error(error: Error) -> ProtocolError {
    match error {
        Error::Protocol(inner) => inner,
        other => panic!("expected protocol error, got {other:?}"),
    }
}

pub fn configured_alarm_device() -> VirtualDevice {
    VirtualDevice::default()
        .with_register(VirtSfr::CrLev1Cp10s, 300)
        .with_register(VirtSfr::CrLev2Cp10s, 600)
        .with_register(VirtSfr::DrLev1UrH, 100)
        .with_register(VirtSfr::DrLev2UrH, 200)
        .with_register(VirtSfr::DsLev1Ur, 1000)
        .with_register(VirtSfr::DsLev2Ur, 2000)
        .with_register(VirtSfr::DsUnits, 1)
        .with_register(VirtSfr::CrUnits, 0)
        .with_register(VirtSfr::AlarmMode, 1)
        .with_register(VirtSfr::DispBrt, 12)
        .with_register(VirtSfr::DispOffTime, 3)
        .with_register(VirtSfr::DispDir, 2)
        .with_register(VirtSfr::SoundOn, 1)
        .with_register(VirtSfr::SoundCtrl, 0x1f9d)
        .with_register(VirtSfr::VibroOn, 0)
        .with_register(VirtSfr::VibroCtrl, 0x1fff)
        .with_register(VirtSfr::DeviceCtrl, 0x35)
}

pub fn spectrum_device() -> VirtualDevice {
    let payload = spectrum_v0_payload(120, [1.0, 2.0, 3.0], &[5, 6, 7]);
    let calibration = [
        1.5f32.to_le_bytes(),
        2.5f32.to_le_bytes(),
        3.5f32.to_le_bytes(),
    ]
    .concat();
    VirtualDevice::default()
        .with_string(VirtString::Configuration, b"SpecFormatVersion=0\n")
        .with_string(VirtString::Spectrum, &payload)
        .with_string(VirtString::SpecAccum, &payload)
        .with_string(VirtString::SpecDiff, &payload)
        .with_string(VirtString::EnergyCalib, &calibration)
}

pub fn zero_alarm_limits() -> AlarmLimits {
    AlarmLimits {
        l1_count_rate: 0.0,
        l2_count_rate: 0.0,
        l1_dose_rate: 0.0,
        l2_dose_rate: 0.0,
        l1_dose: 0.0,
        l2_dose: 0.0,
        dose_unit: DoseDisplayUnit::MicroSievertPerHour,
        count_unit: CountDisplayUnit::Cpm,
    }
}

pub fn frame_with_rare() -> Vec<u8> {
    let mut bytes = real_time_record(1, 100, 5.0, 0.0001);
    bytes.extend(rare_record(2, 200, 87.5, 24.5));
    bytes.extend(real_time_record(3, 300, 6.0, 0.0002));
    bytes
}

pub fn device_with_frame(frame: &[u8]) -> VirtualDevice {
    VirtualDevice::default()
        .with_string(VirtString::DataBuf, frame)
        .with_register(VirtSfr::TempDegC, 31.0f32.to_bits())
}
