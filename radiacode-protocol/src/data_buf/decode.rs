use crate::buffer::BytesBuffer;
use crate::error::{Error, Result};
use crate::units::{CountRateCps, DoseRateRh, DoseRoentgen};

use super::flags::{EventId, StatusFlags};
use super::group::RecordKind;
use super::header::{DeviceTicks, RecordHeader};
use super::records::{
    AccelData, DataBufRecord, DoseRateDb, EventRecord, RareData, RawData, RealTimeData,
};
use super::sanity::{record_passes_sanity, SanityFailure};
use super::seq::Seq;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeWarningKind {
    HeaderUnderrun,
    PayloadUnderrun,
    UnknownRecord { entity: u8, group: u8 },
    SeqJump { expected: u8, got: u8 },
    SanityRejected(SanityFailure),
    TruncatedTail,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodeWarning {
    pub offset: usize,
    pub kind: DecodeWarningKind,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct DataBufFrame {
    pub records: Vec<DataBufRecord>,
    pub warnings: Vec<DecodeWarning>,
}

pub fn decode_data_buf(data: &[u8]) -> DataBufFrame {
    let mut buffer = BytesBuffer::new(data.to_vec());
    let mut frame = DataBufFrame::default();
    let mut expected_seq: Option<Seq> = None;
    while buffer.size() >= 7 {
        let offset = data.len().saturating_sub(buffer.size());
        let Some(header) = parse_header(&mut buffer, offset, &mut frame.warnings) else {
            break;
        };
        if let Some(expected) = expected_seq {
            if header.seq != expected {
                frame.warnings.push(DecodeWarning {
                    offset,
                    kind: DecodeWarningKind::SeqJump {
                        expected: expected.raw(),
                        got: header.seq.raw(),
                    },
                });
            }
        }
        expected_seq = Some(header.seq.next());
        match parse_payload(&mut buffer, header) {
            Ok(record) => {
                if let Err(failure) = record_passes_sanity(&record) {
                    frame.warnings.push(DecodeWarning {
                        offset,
                        kind: DecodeWarningKind::SanityRejected(failure),
                    });
                } else {
                    frame.records.push(record);
                }
            }
            Err(warning) => frame.warnings.push(warning),
        }
    }
    if buffer.size() > 0 {
        frame.warnings.push(DecodeWarning {
            offset: data.len().saturating_sub(buffer.size()),
            kind: DecodeWarningKind::TruncatedTail,
        });
    }
    frame
}

fn parse_header(
    buffer: &mut BytesBuffer,
    offset: usize,
    warnings: &mut Vec<DecodeWarning>,
) -> Option<RecordHeader> {
    let seq = Seq::new(buffer.take_u8().ok()?);
    let entity = buffer.take_u8().ok()?;
    let group = buffer.take_u8().ok()?;
    let ts = DeviceTicks::new(buffer.take_i32_le().ok()?);
    let kind = RecordKind::from_entity_group(entity, group);
    if matches!(kind, RecordKind::Unknown { .. }) {
        warnings.push(DecodeWarning {
            offset,
            kind: DecodeWarningKind::UnknownRecord { entity, group },
        });
    }
    Some(RecordHeader { seq, kind, ts })
}

fn parse_payload(
    buffer: &mut BytesBuffer,
    header: RecordHeader,
) -> std::result::Result<DataBufRecord, DecodeWarning> {
    let offset = buffer.size();
    let result = match header.kind {
        RecordKind::RealTimeData => parse_real_time(buffer, header).map(DataBufRecord::RealTime),
        RecordKind::RawData => parse_raw(buffer, header).map(DataBufRecord::Raw),
        RecordKind::DoseRateDb => parse_dose_rate_db(buffer, header).map(DataBufRecord::DoseRateDb),
        RecordKind::RareData => parse_rare(buffer, header).map(DataBufRecord::Rare),
        RecordKind::AccelData => parse_accel(buffer, header).map(DataBufRecord::Accel),
        RecordKind::Event => parse_event(buffer, header).map(DataBufRecord::Event),
        RecordKind::UserData | RecordKind::ScheduleData => {
            buffer.skip(16).map(|_| DataBufRecord::Skipped(header))
        }
        RecordKind::RawCountRate | RecordKind::RawDoseRate => {
            buffer.skip(6).map(|_| DataBufRecord::Skipped(header))
        }
        RecordKind::Waveform8 | RecordKind::Waveform16 | RecordKind::Waveform14 => {
            skip_waveform(buffer, header.kind).map(|_| DataBufRecord::Skipped(header))
        }
        RecordKind::Unknown { entity, group } => {
            return Err(DecodeWarning {
                offset,
                kind: DecodeWarningKind::UnknownRecord { entity, group },
            });
        }
    };
    result.map_err(|error| payload_warning(offset, error))
}

fn payload_warning(offset: usize, error: Error) -> DecodeWarning {
    DecodeWarning {
        offset,
        kind: match error {
            Error::BufferUnderrun { .. } => DecodeWarningKind::PayloadUnderrun,
            _ => DecodeWarningKind::PayloadUnderrun,
        },
    }
}

fn parse_real_time(buffer: &mut BytesBuffer, header: RecordHeader) -> Result<RealTimeData> {
    let count_rate = CountRateCps::new(buffer.take_f32_le()?);
    let dose_rate = DoseRateRh::new(buffer.take_f32_le()?);
    let count_rate_err = buffer.take_u16_le()?;
    let dose_rate_err = buffer.take_u16_le()?;
    let flags = StatusFlags::new(buffer.take_u16_le()?);
    let real_time_flags = buffer.take_u8()?;
    Ok(RealTimeData {
        header,
        count_rate_cps: count_rate,
        dose_rate_rh: dose_rate,
        count_rate_err_pct: f32::from(count_rate_err) / 10.0,
        dose_rate_err_pct: f32::from(dose_rate_err) / 10.0,
        flags,
        real_time_flags,
    })
}

fn parse_raw(buffer: &mut BytesBuffer, header: RecordHeader) -> Result<RawData> {
    Ok(RawData {
        header,
        count_rate_cps: CountRateCps::new(buffer.take_f32_le()?),
        dose_rate_rh: DoseRateRh::new(buffer.take_f32_le()?),
    })
}

fn parse_dose_rate_db(buffer: &mut BytesBuffer, header: RecordHeader) -> Result<DoseRateDb> {
    let count = buffer.take_u32_le()?;
    let count_rate = CountRateCps::new(buffer.take_f32_le()?);
    let dose_rate = DoseRateRh::new(buffer.take_f32_le()?);
    let dose_rate_err = buffer.take_u16_le()?;
    let flags = StatusFlags::new(buffer.take_u16_le()?);
    Ok(DoseRateDb {
        header,
        count,
        count_rate_cps: count_rate,
        dose_rate_rh: dose_rate,
        dose_rate_err_pct: f32::from(dose_rate_err) / 10.0,
        flags,
    })
}

fn parse_rare(buffer: &mut BytesBuffer, header: RecordHeader) -> Result<RareData> {
    let duration_secs = buffer.take_u32_le()?;
    let dose_r = DoseRoentgen::new(buffer.take_f32_le()?);
    let temperature_raw = buffer.take_u16_le()?;
    let charge_raw = buffer.take_u16_le()?;
    let flags = StatusFlags::new(buffer.take_u16_le()?);
    Ok(RareData {
        header,
        duration_secs,
        dose_r,
        temperature_c: (f32::from(temperature_raw) - 2000.0) / 100.0,
        battery_percent: f32::from(charge_raw) / 100.0,
        flags,
    })
}

fn parse_accel(buffer: &mut BytesBuffer, header: RecordHeader) -> Result<AccelData> {
    Ok(AccelData {
        header,
        x: buffer.take_u16_le()?,
        y: buffer.take_u16_le()?,
        z: buffer.take_u16_le()?,
    })
}

fn parse_event(buffer: &mut BytesBuffer, header: RecordHeader) -> Result<EventRecord> {
    let event = EventId::from_raw(buffer.take_u8()?);
    let event_param1 = buffer.take_u8()?;
    let flags = StatusFlags::new(buffer.take_u16_le()?);
    Ok(EventRecord {
        header,
        event,
        event_param1,
        flags,
    })
}

fn skip_waveform(buffer: &mut BytesBuffer, kind: RecordKind) -> Result<()> {
    let samples = buffer.take_u16_le()? as usize;
    let _sample_time = buffer.take_u32_le()?;
    let sample_bytes = match kind {
        RecordKind::Waveform8 => 8,
        RecordKind::Waveform16 => 16,
        RecordKind::Waveform14 => 14,
        _ => 0,
    };
    buffer.skip(samples.saturating_mul(sample_bytes))
}
