use tracing::debug;

use crate::device::RadiaCode;
use crate::error::Result;
use crate::rate_units::{
    decode_count_alarm, decode_dose_accum, decode_dose_alarm, encode_count_alarm,
    encode_dose_accum, encode_dose_alarm,
};
use crate::types::{AlarmLimits, AlarmLimitsUpdate};
use radiacode_protocol::VirtSfr;
use radiacode_protocol::{
    CountDisplayUnit, DoseDisplayUnit, RawCountsPer10s, RawMicroRoentgenPerHour,
};

pub async fn alarm_limits(device: &mut RadiaCode) -> Result<AlarmLimits> {
    let ids = [
        VirtSfr::CrLev1Cp10s,
        VirtSfr::CrLev2Cp10s,
        VirtSfr::DrLev1UrH,
        VirtSfr::DrLev2UrH,
        VirtSfr::DsLev1Ur,
        VirtSfr::DsLev2Ur,
        VirtSfr::DsUnits,
        VirtSfr::CrUnits,
    ];
    let values = device.read_vsfr_batch(&ids).await?;
    let dose_unit = DoseDisplayUnit::from_device_flag(values[6]);
    let count_unit = CountDisplayUnit::from_device_flag(values[7]);
    let limits = AlarmLimits {
        l1_count_rate: decode_count_alarm(RawCountsPer10s::new(values[0]), count_unit),
        l2_count_rate: decode_count_alarm(RawCountsPer10s::new(values[1]), count_unit),
        l1_dose_rate: decode_dose_alarm(RawMicroRoentgenPerHour::new(values[2]), dose_unit),
        l2_dose_rate: decode_dose_alarm(RawMicroRoentgenPerHour::new(values[3]), dose_unit),
        l1_dose: decode_dose_accum(values[4], dose_unit),
        l2_dose: decode_dose_accum(values[5], dose_unit),
        dose_unit,
        count_unit,
    };
    debug!(?limits, "alarm limits loaded");
    Ok(limits)
}

pub async fn set_alarm_limits(device: &mut RadiaCode, update: &AlarmLimitsUpdate) -> Result<()> {
    let current = alarm_limits(device).await?;
    let dose_unit = update.dose_unit.unwrap_or(current.dose_unit);
    let count_unit = update.count_unit.unwrap_or(current.count_unit);
    let mut pairs = Vec::new();
    if let Some(value) = update.l1_count_rate {
        pairs.push((
            VirtSfr::CrLev1Cp10s,
            encode_count_alarm(value, count_unit).as_u32(),
        ));
    }
    if let Some(value) = update.l2_count_rate {
        pairs.push((
            VirtSfr::CrLev2Cp10s,
            encode_count_alarm(value, count_unit).as_u32(),
        ));
    }
    if let Some(value) = update.l1_dose_rate {
        pairs.push((
            VirtSfr::DrLev1UrH,
            encode_dose_alarm(value, dose_unit).as_u32(),
        ));
    }
    if let Some(value) = update.l2_dose_rate {
        pairs.push((
            VirtSfr::DrLev2UrH,
            encode_dose_alarm(value, dose_unit).as_u32(),
        ));
    }
    if let Some(value) = update.l1_dose {
        pairs.push((VirtSfr::DsLev1Ur, encode_dose_accum(value, dose_unit)));
    }
    if let Some(value) = update.l2_dose {
        pairs.push((VirtSfr::DsLev2Ur, encode_dose_accum(value, dose_unit)));
    }
    if let Some(value) = update.dose_unit {
        pairs.push((VirtSfr::DsUnits, value.to_device_flag()));
    }
    if let Some(value) = update.count_unit {
        pairs.push((VirtSfr::CrUnits, value.to_device_flag()));
    }
    if pairs.is_empty() {
        return Ok(());
    }
    device.write_vsfr_batch(&pairs).await?;
    debug!(count = pairs.len(), "alarm limits written");
    Ok(())
}

impl RadiaCode {
    pub async fn alarm_limits(&mut self) -> Result<AlarmLimits> {
        alarm_limits(self).await
    }

    pub async fn set_alarm_limits(&mut self, update: &AlarmLimitsUpdate) -> Result<()> {
        set_alarm_limits(self, update).await
    }
}
