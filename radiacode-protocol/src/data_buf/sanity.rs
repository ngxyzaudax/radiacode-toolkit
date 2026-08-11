use super::records::{DataBufRecord, DoseRateDb, RareData, RawData, RealTimeData};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SanityFailure {
    NonFiniteRate,
    NegativeRate,
    BatteryOutOfRange,
    TemperatureOutOfRange,
}

pub fn record_passes_sanity(record: &DataBufRecord) -> Result<(), SanityFailure> {
    match record {
        DataBufRecord::RealTime(record) => sanity_real_time(record),
        DataBufRecord::Raw(record) => sanity_raw(record),
        DataBufRecord::DoseRateDb(record) => sanity_dose_rate_db(record),
        DataBufRecord::Rare(record) => sanity_rare(record),
        DataBufRecord::Accel(_) | DataBufRecord::Event(_) | DataBufRecord::Skipped(_) => Ok(()),
    }
}

fn sanity_real_time(record: &RealTimeData) -> Result<(), SanityFailure> {
    sanity_rates(record.count_rate_cps.as_f32(), record.dose_rate_rh.as_f32())
}

fn sanity_raw(record: &RawData) -> Result<(), SanityFailure> {
    sanity_rates(record.count_rate_cps.as_f32(), record.dose_rate_rh.as_f32())
}

fn sanity_dose_rate_db(record: &DoseRateDb) -> Result<(), SanityFailure> {
    sanity_rates(record.count_rate_cps.as_f32(), record.dose_rate_rh.as_f32())
}

fn sanity_rare(record: &RareData) -> Result<(), SanityFailure> {
    if !(0.0..=150.0).contains(&record.battery_percent) {
        return Err(SanityFailure::BatteryOutOfRange);
    }
    if !(-40.0..=85.0).contains(&record.temperature_c) {
        return Err(SanityFailure::TemperatureOutOfRange);
    }
    Ok(())
}

fn sanity_rates(count_rate_cps: f32, dose_rate_rh: f32) -> Result<(), SanityFailure> {
    if !count_rate_cps.is_finite() || !dose_rate_rh.is_finite() {
        return Err(SanityFailure::NonFiniteRate);
    }
    if count_rate_cps < 0.0 || dose_rate_rh < 0.0 {
        return Err(SanityFailure::NegativeRate);
    }
    Ok(())
}
