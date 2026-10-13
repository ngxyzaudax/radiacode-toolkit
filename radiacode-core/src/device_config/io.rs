use tracing::debug;

use radiacode_protocol::{RawCountsPer10s, RawMicroRoentgenPerHour};
use radiacode_protocol::{VirtSfr, sfr_entries_support_leds_on};

use crate::device::RadiaCode;
use crate::error::Result;
use crate::rate_units::{
    CountDisplayUnit, DoseDisplayUnit, decode_count_alarm, decode_dose_accum, decode_dose_alarm,
};
use crate::types::AlarmLimits;

use super::signal_flags::SignalFlags;
use super::types::{
    AlarmSignalMode, BacklightOffTime, DEVICE_CTRL_LIGHT, DeviceConfig, DisplayDirection,
};

pub(crate) const CONFIG_REGISTERS: [VirtSfr; 16] = [
    VirtSfr::CrLev1Cp10s,
    VirtSfr::CrLev2Cp10s,
    VirtSfr::DrLev1UrH,
    VirtSfr::DrLev2UrH,
    VirtSfr::DsLev1Ur,
    VirtSfr::DsLev2Ur,
    VirtSfr::DsUnits,
    VirtSfr::CrUnits,
    VirtSfr::AlarmMode,
    VirtSfr::DispBrt,
    VirtSfr::DispOffTime,
    VirtSfr::DispDir,
    VirtSfr::SoundOn,
    VirtSfr::SoundCtrl,
    VirtSfr::VibroOn,
    VirtSfr::VibroCtrl,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct LightState {
    pub leds_on: bool,
    pub leds_supported: bool,
    pub leds_uses_device_ctrl: bool,
}

pub async fn load_device_config(device: &mut RadiaCode) -> Result<DeviceConfig> {
    let values = device.read_vsfr_batch(&CONFIG_REGISTERS).await?;
    let light = load_light_state(device).await?;
    let config = device_config_from_words(&values, light);
    debug!(?config, "device config loaded");
    Ok(config)
}

pub(crate) fn alarm_limits_from_words(values: &[u32]) -> AlarmLimits {
    let dose_unit = DoseDisplayUnit::from_device_flag(values[6]);
    let count_unit = CountDisplayUnit::from_device_flag(values[7]);
    AlarmLimits {
        l1_count_rate: decode_count_alarm(RawCountsPer10s::new(values[0]), count_unit),
        l2_count_rate: decode_count_alarm(RawCountsPer10s::new(values[1]), count_unit),
        l1_dose_rate: decode_dose_alarm(RawMicroRoentgenPerHour::new(values[2]), dose_unit),
        l2_dose_rate: decode_dose_alarm(RawMicroRoentgenPerHour::new(values[3]), dose_unit),
        l1_dose: decode_dose_accum(values[4], dose_unit),
        l2_dose: decode_dose_accum(values[5], dose_unit),
        dose_unit,
        count_unit,
    }
}

fn device_config_from_words(values: &[u32], light: LightState) -> DeviceConfig {
    DeviceConfig {
        alarms: alarm_limits_from_words(values),
        alarm_mode: AlarmSignalMode::from_raw(values[8]),
        brightness: values[9].min(9) as u8,
        backlight_off: BacklightOffTime::from_raw(values[10]),
        display_dir: DisplayDirection::from_raw(values[11]),
        sound_on: values[12] != 0,
        sound_ctrl: SignalFlags::from_raw(values[13]),
        vibro_on: values[14] != 0,
        vibro_ctrl: SignalFlags::from_raw(values[15]).vibro_events(),
        leds_on: light.leds_on,
        leds_supported: light.leds_supported,
        leds_uses_device_ctrl: light.leds_uses_device_ctrl,
    }
}

async fn load_light_state(device: &mut RadiaCode) -> Result<LightState> {
    if sfr_entries_support_leds_on(device.sfr_entries()) {
        let leds_on = device
            .read_vsfr_optional(VirtSfr::LedsOn)
            .await?
            .is_some_and(|value| value != 0);
        return Ok(LightState {
            leds_on,
            leds_supported: true,
            leds_uses_device_ctrl: false,
        });
    }
    let device_ctrl = device.read_vsfr_u32(VirtSfr::DeviceCtrl).await?;
    Ok(LightState {
        leds_on: device_ctrl & DEVICE_CTRL_LIGHT != 0,
        leds_supported: true,
        leds_uses_device_ctrl: true,
    })
}
