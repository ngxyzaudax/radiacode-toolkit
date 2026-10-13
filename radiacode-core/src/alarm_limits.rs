use tracing::debug;

use crate::device::RadiaCode;
use crate::device_config::{CONFIG_REGISTERS, alarm_limits_from_words, alarm_register_pairs};
use crate::error::Result;
use crate::types::{AlarmLimits, AlarmLimitsUpdate};
use radiacode_protocol::VirtSfr;

const ALARM_REGISTER_COUNT: usize = 8;

pub async fn alarm_limits(device: &mut RadiaCode) -> Result<AlarmLimits> {
    let values = device
        .read_vsfr_batch(&CONFIG_REGISTERS[..ALARM_REGISTER_COUNT])
        .await?;
    let limits = alarm_limits_from_words(&values);
    debug!(?limits, "alarm limits loaded");
    Ok(limits)
}

pub async fn set_alarm_limits(device: &mut RadiaCode, update: &AlarmLimitsUpdate) -> Result<()> {
    let current = alarm_limits(device).await?;
    let pairs = updated_alarm_pairs(&current, update);
    if pairs.is_empty() {
        return Ok(());
    }
    device.write_vsfr_batch(&pairs).await?;
    debug!(count = pairs.len(), "alarm limits written");
    Ok(())
}

pub(crate) fn merge_alarm_update(current: &AlarmLimits, update: &AlarmLimitsUpdate) -> AlarmLimits {
    AlarmLimits {
        l1_count_rate: update.l1_count_rate.unwrap_or(current.l1_count_rate),
        l2_count_rate: update.l2_count_rate.unwrap_or(current.l2_count_rate),
        l1_dose_rate: update.l1_dose_rate.unwrap_or(current.l1_dose_rate),
        l2_dose_rate: update.l2_dose_rate.unwrap_or(current.l2_dose_rate),
        l1_dose: update.l1_dose.unwrap_or(current.l1_dose),
        l2_dose: update.l2_dose.unwrap_or(current.l2_dose),
        dose_unit: update.dose_unit.unwrap_or(current.dose_unit),
        count_unit: update.count_unit.unwrap_or(current.count_unit),
    }
}

fn provided_fields(update: &AlarmLimitsUpdate) -> [bool; ALARM_REGISTER_COUNT] {
    [
        update.l1_count_rate.is_some(),
        update.l2_count_rate.is_some(),
        update.l1_dose_rate.is_some(),
        update.l2_dose_rate.is_some(),
        update.l1_dose.is_some(),
        update.l2_dose.is_some(),
        update.dose_unit.is_some(),
        update.count_unit.is_some(),
    ]
}

pub(crate) fn updated_alarm_pairs(
    current: &AlarmLimits,
    update: &AlarmLimitsUpdate,
) -> Vec<(VirtSfr, u32)> {
    let merged = merge_alarm_update(current, update);
    alarm_register_pairs(&merged)
        .into_iter()
        .zip(provided_fields(update))
        .filter_map(|(pair, provided)| provided.then_some(pair))
        .collect()
}

impl RadiaCode {
    pub async fn alarm_limits(&mut self) -> Result<AlarmLimits> {
        alarm_limits(self).await
    }

    pub async fn set_alarm_limits(&mut self, update: &AlarmLimitsUpdate) -> Result<()> {
        set_alarm_limits(self, update).await
    }
}
