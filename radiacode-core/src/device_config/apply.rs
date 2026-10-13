use tracing::debug;

use radiacode_protocol::VirtSfr;

use crate::device::RadiaCode;
use crate::device_time::set_local_time_now;
use crate::error::Result;
use crate::rate_units::{encode_count_alarm, encode_dose_accum, encode_dose_alarm};
use crate::types::AlarmLimits;

use super::types::{DEVICE_CTRL_LIGHT, DeviceConfig};

pub async fn apply_device_config(device: &mut RadiaCode, config: &DeviceConfig) -> Result<()> {
    let pairs = config_register_pairs(config);
    device.write_vsfr_batch(&pairs).await?;
    if config.leds_supported && config.leds_uses_device_ctrl {
        apply_device_ctrl_light(device, config.leds_on).await?;
    }
    debug!(count = pairs.len(), "device config applied");
    Ok(())
}

pub(crate) fn alarm_register_pairs(alarms: &AlarmLimits) -> Vec<(VirtSfr, u32)> {
    let dose_unit = alarms.dose_unit;
    let count_unit = alarms.count_unit;
    vec![
        (
            VirtSfr::CrLev1Cp10s,
            encode_count_alarm(alarms.l1_count_rate, count_unit).as_u32(),
        ),
        (
            VirtSfr::CrLev2Cp10s,
            encode_count_alarm(alarms.l2_count_rate, count_unit).as_u32(),
        ),
        (
            VirtSfr::DrLev1UrH,
            encode_dose_alarm(alarms.l1_dose_rate, dose_unit).as_u32(),
        ),
        (
            VirtSfr::DrLev2UrH,
            encode_dose_alarm(alarms.l2_dose_rate, dose_unit).as_u32(),
        ),
        (
            VirtSfr::DsLev1Ur,
            encode_dose_accum(alarms.l1_dose, dose_unit),
        ),
        (
            VirtSfr::DsLev2Ur,
            encode_dose_accum(alarms.l2_dose, dose_unit),
        ),
        (VirtSfr::DsUnits, dose_unit.to_device_flag()),
        (VirtSfr::CrUnits, count_unit.to_device_flag()),
    ]
}

fn config_register_pairs(config: &DeviceConfig) -> Vec<(VirtSfr, u32)> {
    let mut pairs = alarm_register_pairs(&config.alarms);
    pairs.extend([
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
    ]);
    if config.leds_supported && !config.leds_uses_device_ctrl {
        pairs.push((VirtSfr::LedsOn, u32::from(config.leds_on)));
    }
    pairs
}

async fn apply_device_ctrl_light(device: &mut RadiaCode, leds_on: bool) -> Result<()> {
    let current = device.read_vsfr_u32(VirtSfr::DeviceCtrl).await?;
    let next = with_light_bit(current, leds_on);
    if next != current {
        device
            .write_vsfr(VirtSfr::DeviceCtrl, &next.to_le_bytes())
            .await?;
    }
    Ok(())
}

fn with_light_bit(device_ctrl: u32, leds_on: bool) -> u32 {
    if leds_on {
        device_ctrl | DEVICE_CTRL_LIGHT
    } else {
        device_ctrl & !DEVICE_CTRL_LIGHT
    }
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
        super::io::load_device_config(self).await
    }

    pub async fn apply_device_config(&mut self, config: &DeviceConfig) -> Result<()> {
        apply_device_config(self, config).await
    }

    pub async fn sync_device_clock(&mut self) -> Result<()> {
        sync_device_clock(self).await
    }
}
