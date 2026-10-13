mod decode;
mod flags;
mod group;
mod header;
mod records;
mod sanity;
mod seq;
mod snapshot;

#[cfg(test)]
mod event_tests;
#[cfg(test)]
mod flags_tests;
#[cfg(test)]
mod record_payload_tests;
#[cfg(test)]
mod record_tests;
#[cfg(test)]
mod tests;

pub use decode::{DataBufFrame, DecodeWarning, DecodeWarningKind, decode_data_buf};
pub use flags::{EventId, StatusFlags};
pub use group::RecordKind;
pub use header::{DeviceTicks, RecordHeader};
pub use records::{
    AccelData, DataBufRecord, DoseRateDb, EventRecord, RareData, RareStatus, RawData, RealTimeData,
    RealTimeRates,
};
pub use sanity::SanityFailure;
pub use seq::Seq;
pub use snapshot::{
    DataBufSnapshot, latest_rare_record, latest_snapshot, real_time_records, snapshot_from_frame,
};
