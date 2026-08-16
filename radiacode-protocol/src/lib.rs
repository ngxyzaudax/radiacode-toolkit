//! RadiaCode wire protocol: framing, opcodes, payload decoders, and typed units.

mod buffer;
mod catalog;
mod command;
mod data_buf;
mod error;
mod fw_version;
mod protocol;
#[cfg(test)]
mod protocol_tests;
mod rate_units;
mod sfr_catalog;
mod spectrum;
#[cfg(test)]
mod spectrum_tests;
mod transport;
mod types;
mod units;

pub use buffer::BytesBuffer;
pub use catalog::{
    CatalogDrift, ChannelDef, ConfigurationCatalog, MessageGroup, SfrCatalogEntry, SfrValueKind,
    parse_configuration_ini, parse_sfr_file, validate_catalog,
};
pub use command::{Command, VirtSfr, VirtString};
pub use data_buf::{
    AccelData, DataBufFrame, DataBufRecord, DataBufSnapshot, DecodeWarning, DecodeWarningKind,
    DeviceTicks, DoseRateDb, EventId, EventRecord, RareData, RareStatus, RawData, RealTimeData,
    RealTimeRates, RecordHeader, RecordKind, SanityFailure, Seq, StatusFlags, decode_data_buf,
    latest_rare_record, latest_rare_status, latest_real_time_rates, latest_snapshot,
    real_time_records, snapshot_from_frame,
};
pub use error::{Error, Result};
pub use fw_version::decode_fw_version;
pub use protocol::{
    ResponseAssembler, Sequence, build_request, framed_request_header, request_header,
    response_matches_request, strip_echoed_header,
};
pub use rate_units::{
    count_display_from_cps, decode_count_alarm, decode_dose_accum, decode_dose_alarm,
    dose_display_from_rh, encode_count_alarm, encode_dose_accum, encode_dose_alarm,
};
pub use sfr_catalog::sfr_supports_leds_on;
pub use spectrum::decode_spectrum;
pub use transport::Transport;
pub use types::{DeviceVersions, FirmwareVersion, Spectrum, channel_to_energy};
pub use units::{
    CountDisplayUnit, CountRateCps, DoseDisplayUnit, DoseRateRh, DoseRoentgen, RawCountsPer10s,
    RawMicroRoentgenPerHour,
};
