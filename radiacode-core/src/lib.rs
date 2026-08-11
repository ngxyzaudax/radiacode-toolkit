mod alarm_limits;
mod data_buf_cursor;
mod device;
mod device_config;
mod device_info;
mod device_model;
mod device_settings;
mod device_status;
mod device_time;
mod discovery;
mod error;
mod live_rates;
mod metadata;
mod monitor_poll;
mod rate_units;
mod session_restore;
mod status_read;
mod types;
mod vsfr_batch;

pub use data_buf_cursor::{DataBufCursor, SeqGap};
pub use device::RadiaCode;
pub use device_config::{
    AlarmSignalMode, BacklightOffTime, DeviceConfig, DisplayDirection, SignalFlags,
    apply_device_config, load_device_config, sync_device_clock,
};
pub use device_model::{model_from_advertisement, model_from_serial, serial_from_advertisement};
pub use discovery::{
    DeviceEndpoint, DiscoveredDevice, TransportKind, merge_discovered, resolve_usb_endpoint,
};
pub use error::{Error, Result, protocol_error};
pub use radiacode_protocol::{
    BytesBuffer, Command, CountDisplayUnit, DataBufFrame, DataBufRecord, DataBufSnapshot,
    DeviceTicks, DeviceVersions, DoseDisplayUnit, FirmwareVersion, RareStatus, RealTimeData,
    RealTimeRates, RecordKind, ResponseAssembler, Sequence, Spectrum, Transport, VirtSfr,
    VirtString, build_request, channel_to_energy, decode_data_buf, decode_spectrum,
    framed_request_header, response_matches_request, strip_echoed_header,
};
pub use radiacode_protocol::{
    count_display_from_cps, decode_count_alarm, decode_dose_accum, decode_dose_alarm,
    dose_display_from_accum_r, dose_display_from_rh, encode_count_alarm, encode_dose_accum,
    encode_dose_alarm, latest_rare_status, latest_real_time_rates, latest_snapshot,
    parse_configuration_ini, parse_sfr_file, sfr_supports_leds_on, validate_catalog,
};
pub use rate_units::{count_unit_label, dose_accum_unit_label, dose_unit_label};
pub use session_restore::SessionRestore;
pub use status_read::merge_status;
pub use types::{
    AccumulatedDose, AlarmLimits, AlarmLimitsUpdate, DeviceMetadata, DeviceStatus, LiveRates,
    MonitorPollSample, TimedRates,
};
