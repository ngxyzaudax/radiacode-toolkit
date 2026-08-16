use crate::units::{CountRateCps, DoseRateRh, DoseRoentgen};

use super::flags::{EventId, StatusFlags};
use super::header::RecordHeader;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RealTimeData {
    pub header: RecordHeader,
    pub count_rate_cps: CountRateCps,
    pub dose_rate_rh: DoseRateRh,
    pub count_rate_err_pct: f32,
    pub dose_rate_err_pct: f32,
    pub flags: StatusFlags,
    pub real_time_flags: u8,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RawData {
    pub header: RecordHeader,
    pub count_rate_cps: CountRateCps,
    pub dose_rate_rh: DoseRateRh,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DoseRateDb {
    pub header: RecordHeader,
    pub count: u32,
    pub count_rate_cps: CountRateCps,
    pub dose_rate_rh: DoseRateRh,
    pub dose_rate_err_pct: f32,
    pub flags: StatusFlags,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RareData {
    pub header: RecordHeader,
    pub duration_secs: u32,
    pub dose_r: DoseRoentgen,
    pub temperature_c: f32,
    pub battery_percent: f32,
    pub flags: StatusFlags,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AccelData {
    pub header: RecordHeader,
    pub x: u16,
    pub y: u16,
    pub z: u16,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EventRecord {
    pub header: RecordHeader,
    pub event: EventId,
    pub event_param1: u8,
    pub flags: StatusFlags,
    pub value: f32,
    pub trailing: u16,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DataBufRecord {
    RealTime(RealTimeData),
    Raw(RawData),
    DoseRateDb(DoseRateDb),
    Rare(RareData),
    Accel(AccelData),
    Event(EventRecord),
    Skipped(RecordHeader),
}

impl DataBufRecord {
    pub fn header(&self) -> RecordHeader {
        match self {
            Self::RealTime(record) => record.header,
            Self::Raw(record) => record.header,
            Self::DoseRateDb(record) => record.header,
            Self::Rare(record) => record.header,
            Self::Accel(record) => record.header,
            Self::Event(record) => record.header,
            Self::Skipped(header) => *header,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RareStatus {
    pub duration_secs: u32,
    pub dose_r: f32,
    pub temperature_c: f32,
    pub battery_percent: f32,
}

impl From<RareData> for RareStatus {
    fn from(value: RareData) -> Self {
        Self {
            duration_secs: value.duration_secs,
            dose_r: value.dose_r.as_f32(),
            temperature_c: value.temperature_c,
            battery_percent: value.battery_percent,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RealTimeRates {
    pub count_rate_cps: f32,
    pub dose_rate_rh: f32,
    pub count_rate_err_pct: f32,
    pub dose_rate_err_pct: f32,
}

impl From<RealTimeData> for RealTimeRates {
    fn from(value: RealTimeData) -> Self {
        Self {
            count_rate_cps: value.count_rate_cps.as_f32(),
            dose_rate_rh: value.dose_rate_rh.as_f32(),
            count_rate_err_pct: value.count_rate_err_pct,
            dose_rate_err_pct: value.dose_rate_err_pct,
        }
    }
}
