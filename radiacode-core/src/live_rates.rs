use tracing::debug;

use radiacode_protocol::{decode_data_buf, latest_snapshot, RealTimeRates, VirtString};
use crate::device::RadiaCode;
use crate::error::{Error, Result};
use crate::rate_units::{count_display_from_cps, dose_display_from_rh};
use crate::types::{AlarmLimits, LiveRates};

pub async fn live_rates(device: &mut RadiaCode, units: &AlarmLimits) -> Result<LiveRates> {
    let response = device.read_virt_string(VirtString::DataBuf).await?;
    let frame = decode_data_buf(response.data());
    let snapshot = latest_snapshot(response.data());
    let rates = snapshot
        .rates
        .ok_or(Error::MonitorDataPending)
        .map(|raw| to_live_rates(&raw, units))?;
    debug!(warnings = frame.warnings.len(), ?rates, "live rates from databuf");
    Ok(rates)
}

fn to_live_rates(rates: &RealTimeRates, units: &AlarmLimits) -> LiveRates {
    LiveRates {
        dose_rate: dose_display_from_rh(rates.dose_rate_rh, units.dose_unit),
        count_rate: count_display_from_cps(rates.count_rate_cps, units.count_unit),
        dose_unit: units.dose_unit,
        count_unit: units.count_unit,
        dose_rate_err_pct: rates.dose_rate_err_pct,
        count_rate_err_pct: rates.count_rate_err_pct,
    }
}

impl RadiaCode {
    pub async fn live_rates(&mut self, units: &AlarmLimits) -> Result<LiveRates> {
        live_rates(self, units).await
    }
}
