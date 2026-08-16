use std::sync::Once;

use tracing::{debug, warn};

use crate::data_buf_cursor::DataBufCursor;
use crate::device::RadiaCode;
use crate::error::Result;
use crate::rate_units::{count_display_from_cps, dose_display_from_rh};
use crate::status_read::status_from_frame;
use crate::types::{AccumulatedDose, AlarmLimits, DeviceStatus, MonitorPollSample, TimedRates};
use radiacode_protocol::{
    VirtString, decode_data_buf, latest_rare_record, real_time_records, snapshot_from_frame,
};

pub async fn poll_monitor(
    device: &mut RadiaCode,
    units: &AlarmLimits,
    cursor: &mut DataBufCursor,
    refresh_rssi: bool,
) -> Result<(MonitorPollSample, DeviceStatus)> {
    let response = device.read_virt_string(VirtString::DataBuf).await?;
    let frame = decode_data_buf(response.data());
    let seq_gaps = cursor.observe_frame(&frame);
    let snapshot = snapshot_from_frame(&frame);
    let status = status_from_frame(device, &snapshot, refresh_rssi).await?;
    let rates = real_time_records(&frame)
        .map(|record| TimedRates {
            device_ts: record.header.ts,
            dose_rate: dose_display_from_rh(record.dose_rate_rh.as_f32(), units.dose_unit),
            count_rate: count_display_from_cps(record.count_rate_cps.as_f32(), units.count_unit),
            dose_rate_err_pct: record.dose_rate_err_pct,
            count_rate_err_pct: record.count_rate_err_pct,
            dose_unit: units.dose_unit,
            count_unit: units.count_unit,
        })
        .collect();
    let rejected_records = frame
        .warnings
        .iter()
        .filter(|warning| {
            matches!(
                warning.kind,
                radiacode_protocol::DecodeWarningKind::SanityRejected(_)
            )
        })
        .count();
    let decode_warnings = frame
        .warnings
        .iter()
        .filter(|warning| {
            !matches!(
                warning.kind,
                radiacode_protocol::DecodeWarningKind::SeqJump { .. }
                    | radiacode_protocol::DecodeWarningKind::SanityRejected(_)
            )
        })
        .count();
    log_first_unknown_record(&frame);
    let accumulated = latest_rare_record(&frame).map(|rare| AccumulatedDose {
        dose: dose_display_from_rh(rare.dose_r.as_f32(), units.dose_unit),
        duration_secs: rare.duration_secs,
        dose_unit: units.dose_unit,
    });
    let sample = MonitorPollSample {
        rates,
        accumulated,
        decode_warnings,
        rejected_records,
        resync_count: frame.resync_count as usize,
        seq_gaps,
    };
    debug!(
        rate_count = sample.rates.len(),
        decode_warnings = sample.decode_warnings,
        rejected_records = sample.rejected_records,
        resync_count = sample.resync_count,
        seq_gap_count = sample.seq_gaps.len(),
        ?status,
        "monitor poll"
    );
    Ok((sample, status))
}

static UNKNOWN_RECORD_HEX_LOG: Once = Once::new();

fn log_first_unknown_record(frame: &radiacode_protocol::DataBufFrame) {
    for warning in &frame.warnings {
        let radiacode_protocol::DecodeWarningKind::UnknownRecord {
            entity,
            group,
            tail,
        } = &warning.kind
        else {
            continue;
        };
        UNKNOWN_RECORD_HEX_LOG.call_once(|| {
            warn!(
                entity,
                group,
                offset = warning.offset,
                tail_hex = %hex_encode(tail),
                "unknown databuf record type (first occurrence)"
            );
        });
        return;
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .join("")
}

impl RadiaCode {
    pub async fn poll_monitor(
        &mut self,
        units: &AlarmLimits,
        cursor: &mut DataBufCursor,
        refresh_rssi: bool,
    ) -> Result<(MonitorPollSample, DeviceStatus)> {
        poll_monitor(self, units, cursor, refresh_rssi).await
    }
}
