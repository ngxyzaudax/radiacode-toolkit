use tracing::debug;

use radiacode_protocol::{RawCountsPer10s, RawMicroRoentgenPerHour};
use radiacode_protocol::{VirtSfr, VirtString, sfr_supports_leds_on};

use crate::device::RadiaCode;
use crate::device_time::set_local_time_now;
use crate::error::Result;
use crate::rate_units::{
    CountDisplayUnit, DoseDisplayUnit, decode_count_alarm, decode_dose_accum, decode_dose_alarm,
    encode_count_alarm, encode_dose_accum, encode_dose_alarm,
};
use crate::types::AlarmLimits;

use super::types::{
    AlarmSignalMode, BacklightOffTime, DEVICE_CTRL_LIGHT, DeviceConfig, DisplayDirection,
    SignalFlags,
};

pub async fn load_device_config(device: &mut RadiaCode) -> Result<DeviceConfig> {
    let ids = [
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
    let values = device.read_vsfr_batch(&ids).await?;
    let dose_unit = DoseDisplayUnit::from_device_flag(values[6]);
    let count_unit = CountDisplayUnit::from_device_flag(values[7]);
    let (leds_on, leds_supported, leds_uses_device_ctrl) = load_light_state(device).await?;
    let config = DeviceConfig {
        alarms: AlarmLimits {
            l1_count_rate: decode_count_alarm(RawCountsPer10s::new(values[0]), count_unit),
            l2_count_rate: decode_count_alarm(RawCountsPer10s::new(values[1]), count_unit),
            l1_dose_rate: decode_dose_alarm(RawMicroRoentgenPerHour::new(values[2]), dose_unit),
            l2_dose_rate: decode_dose_alarm(RawMicroRoentgenPerHour::new(values[3]), dose_unit),
            l1_dose: decode_dose_accum(values[4], dose_unit),
            l2_dose: decode_dose_accum(values[5], dose_unit),
            dose_unit,
            count_unit,
        },
        alarm_mode: AlarmSignalMode::from_raw(values[8]),
        brightness: values[9].min(9) as u8,
        backlight_off: BacklightOffTime::from_raw(values[10]),
        display_dir: DisplayDirection::from_raw(values[11]),
        sound_on: values[12] != 0,
        sound_ctrl: SignalFlags::from_raw(values[13]),
        vibro_on: values[14] != 0,
        vibro_ctrl: SignalFlags::from_raw(values[15]).vibro_events(),
        leds_on,
        leds_supported,
        leds_uses_device_ctrl,
    };
    debug!(?config, "device config loaded");
    Ok(config)
}

async fn load_light_state(device: &mut RadiaCode) -> Result<(bool, bool, bool)> {
    let sfr_file = device.read_virt_string(VirtString::SfrFile).await?;
    let has_leds_on = sfr_supports_leds_on(&String::from_utf8_lossy(sfr_file.data()));
    if has_leds_on {
        let leds_on = device
            .read_vsfr_optional(VirtSfr::LedsOn)
            .await?
            .map(|value| value != 0)
            .unwrap_or(false);
        return Ok((leds_on, true, false));
    }
    let device_ctrl = device.read_vsfr_u32(VirtSfr::DeviceCtrl).await?;
    let leds_on = device_ctrl & DEVICE_CTRL_LIGHT != 0;
    Ok((leds_on, true, true))
}

pub async fn apply_device_config(device: &mut RadiaCode, config: &DeviceConfig) -> Result<()> {
    let dose_unit = config.alarms.dose_unit;
    let count_unit = config.alarms.count_unit;
    let mut pairs = vec![
        (
            VirtSfr::CrLev1Cp10s,
            encode_count_alarm(config.alarms.l1_count_rate, count_unit).as_u32(),
        ),
        (
            VirtSfr::CrLev2Cp10s,
            encode_count_alarm(config.alarms.l2_count_rate, count_unit).as_u32(),
        ),
        (
            VirtSfr::DrLev1UrH,
            encode_dose_alarm(config.alarms.l1_dose_rate, dose_unit).as_u32(),
        ),
        (
            VirtSfr::DrLev2UrH,
            encode_dose_alarm(config.alarms.l2_dose_rate, dose_unit).as_u32(),
        ),
        (
            VirtSfr::DsLev1Ur,
            encode_dose_accum(config.alarms.l1_dose, dose_unit),
        ),
        (
            VirtSfr::DsLev2Ur,
            encode_dose_accum(config.alarms.l2_dose, dose_unit),
        ),
        (VirtSfr::DsUnits, dose_unit.to_device_flag()),
        (VirtSfr::CrUnits, count_unit.to_device_flag()),
        (VirtSfr::AlarmMode, config.alarm_mode.as_raw()),
        (VirtSfr::DispBrt, u32::from(config.brightness.min(9))),
        (VirtSfr::DispOffTime, config.backlight_off.as_raw()),
        (VirtSfr::DispDir, config.display_dir.as_raw()),
        (VirtSfr::SoundOn, u32::from(config.sound_on)),
        (VirtSfr::SoundCtrl, config.sound_ctrl.as_raw()),
        (VirtSfr::VibroOn, u32::from(config.vibro_on)),
        (
            VirtSfr::VibroCtrl,
            config.vibro_ctrl.vibro_events().as_raw(),
        ),
    ];
    if config.leds_supported && !config.leds_uses_device_ctrl {
        pairs.push((VirtSfr::LedsOn, u32::from(config.leds_on)));
    }
    device.write_vsfr_batch(&pairs).await?;
    if config.leds_supported && config.leds_uses_device_ctrl {
        apply_device_ctrl_light(device, config.leds_on).await?;
    }
    debug!(count = pairs.len(), "device config applied");
    Ok(())
}

async fn apply_device_ctrl_light(device: &mut RadiaCode, leds_on: bool) -> Result<()> {
    let current = device.read_vsfr_u32(VirtSfr::DeviceCtrl).await?;
    let next = if leds_on {
        current | DEVICE_CTRL_LIGHT
    } else {
        current & !DEVICE_CTRL_LIGHT
    };
    if next != current {
        device
            .write_vsfr(VirtSfr::DeviceCtrl, &next.to_le_bytes())
            .await?;
    }
    Ok(())
}

pub async fn sync_device_clock(device: &mut RadiaCode) -> Result<()> {
    set_local_time_now(device).await?;
    device
        .write_vsfr(VirtSfr::DeviceTime, &0u32.to_le_bytes())
        .await?;
    Ok(())
}

impl RadiaCode {
    pub async fn load_device_config(&mut self) -> Result<DeviceConfig> {
        load_device_config(self).await
    }

    pub async fn apply_device_config(&mut self, config: &DeviceConfig) -> Result<()> {
        apply_device_config(self, config).await
    }

    pub async fn sync_device_clock(&mut self) -> Result<()> {
        sync_device_clock(self).await
    }
}
