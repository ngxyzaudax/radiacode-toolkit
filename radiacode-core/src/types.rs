use radiacode_protocol::{CountDisplayUnit, DeviceTicks, DoseDisplayUnit};

#[derive(Debug, Clone, PartialEq)]
pub struct DeviceMetadata {
    pub serial: String,
    pub model: String,
    pub versions: radiacode_protocol::DeviceVersions,
    pub energy_calib: [f32; 3],
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct DeviceStatus {
    pub battery_percent: Option<f32>,
    pub temperature_c: Option<f32>,
    pub rssi_dbm: Option<i16>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AlarmLimits {
    pub l1_count_rate: f32,
    pub l2_count_rate: f32,
    pub l1_dose_rate: f32,
    pub l2_dose_rate: f32,
    pub l1_dose: f32,
    pub l2_dose: f32,
    pub dose_unit: DoseDisplayUnit,
    pub count_unit: CountDisplayUnit,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct AlarmLimitsUpdate {
    pub l1_count_rate: Option<f32>,
    pub l2_count_rate: Option<f32>,
    pub l1_dose_rate: Option<f32>,
    pub l2_dose_rate: Option<f32>,
    pub l1_dose: Option<f32>,
    pub l2_dose: Option<f32>,
    pub dose_unit: Option<DoseDisplayUnit>,
    pub count_unit: Option<CountDisplayUnit>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LiveRates {
    pub dose_rate: f32,
    pub count_rate: f32,
    pub dose_unit: DoseDisplayUnit,
    pub count_unit: CountDisplayUnit,
    pub dose_rate_err_pct: f32,
    pub count_rate_err_pct: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimedRates {
    pub device_ts: DeviceTicks,
    pub dose_rate: f32,
    pub count_rate: f32,
    pub dose_rate_err_pct: f32,
    pub count_rate_err_pct: f32,
    pub dose_unit: DoseDisplayUnit,
    pub count_unit: CountDisplayUnit,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AccumulatedDose {
    pub dose: f32,
    pub duration_secs: u32,
    pub dose_unit: DoseDisplayUnit,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MonitorPollSample {
    pub rates: Vec<TimedRates>,
    pub accumulated: Option<AccumulatedDose>,
    pub decode_warnings: usize,
    pub rejected_records: usize,
    pub seq_gaps: Vec<crate::data_buf_cursor::SeqGap>,
}
