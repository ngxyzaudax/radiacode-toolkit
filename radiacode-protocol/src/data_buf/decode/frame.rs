use crate::buffer::BytesBuffer;

use super::super::group::RecordKind;
use super::super::header::{DeviceTicks, RecordHeader};
use super::super::records::DataBufRecord;
use super::super::sanity::record_passes_sanity;
use super::super::seq::Seq;
use super::records::parse_payload;

const MAX_RESYNCS_PER_FRAME: u32 = 4;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeWarningKind {
    HeaderUnderrun,
    PayloadUnderrun,
    UnknownRecord {
        entity: u8,
        group: u8,
        tail: Vec<u8>,
    },
    SeqJump {
        expected: u8,
        got: u8,
    },
    SanityRejected(super::super::sanity::SanityFailure),
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
    pub first_header_seq: Option<Seq>,
    pub last_header_seq: Option<Seq>,
    pub resync_count: u32,
}

pub fn decode_data_buf(data: &[u8]) -> DataBufFrame {
    let mut buffer = BytesBuffer::new(data.to_vec());
    let mut frame = DataBufFrame::default();
    let mut expected_seq: Option<Seq> = None;
    while buffer.size() >= 7 {
        let offset = data.len().saturating_sub(buffer.size());
        let Some(header) = parse_header(&mut buffer, offset) else {
            break;
        };
        if frame.first_header_seq.is_none() {
            frame.first_header_seq = Some(header.seq);
        }
        frame.last_header_seq = Some(header.seq);
        if matches!(header.kind, RecordKind::Unknown { .. }) {
            let RecordKind::Unknown { entity, group } = header.kind else {
                unreachable!();
            };
            push_unknown_warning(&mut frame, offset, entity, group, data, &buffer);
            let Some(expected) = expected_seq else {
                break;
            };
            if try_resync(&mut buffer, expected, &mut frame) {
                continue;
            }
            break;
        }
        if let Some(expected) = expected_seq
            && header.seq != expected
        {
            frame.warnings.push(DecodeWarning {
                offset,
                kind: DecodeWarningKind::SeqJump {
                    expected: expected.raw(),
                    got: header.seq.raw(),
                },
            });
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
            Err(warning) => {
                frame.warnings.push(warning);
                break;
            }
        }
    }
    if buffer.size() > 0
        && !frame
            .warnings
            .last()
            .is_some_and(|warning| matches!(warning.kind, DecodeWarningKind::UnknownRecord { .. }))
    {
        frame.warnings.push(DecodeWarning {
            offset: data.len().saturating_sub(buffer.size()),
            kind: DecodeWarningKind::TruncatedTail,
        });
    }
    frame
}

fn parse_header(buffer: &mut BytesBuffer, offset: usize) -> Option<RecordHeader> {
    let _ = offset;
    let seq = Seq::new(buffer.take_u8().ok()?);
    let entity = buffer.take_u8().ok()?;
    let group = buffer.take_u8().ok()?;
    let ts = DeviceTicks::new(buffer.take_i32_le().ok()?);
    let kind = RecordKind::from_entity_group(entity, group);
    Some(RecordHeader { seq, kind, ts })
}

fn push_unknown_warning(
    frame: &mut DataBufFrame,
    offset: usize,
    entity: u8,
    group: u8,
    data: &[u8],
    buffer: &BytesBuffer,
) {
    let tail_start = data.len().saturating_sub(buffer.size());
    let tail_end = tail_start.saturating_add(32).min(data.len());
    frame.warnings.push(DecodeWarning {
        offset,
        kind: DecodeWarningKind::UnknownRecord {
            entity,
            group,
            tail: data[tail_start..tail_end].to_vec(),
        },
    });
}

fn try_resync(buffer: &mut BytesBuffer, expected_seq: Seq, frame: &mut DataBufFrame) -> bool {
    if frame.resync_count >= MAX_RESYNCS_PER_FRAME {
        return false;
    }
    let remaining = buffer.data();
    if remaining.len() < 7 {
        return false;
    }
    for skip in 1..remaining.len().saturating_sub(6) {
        let seq = remaining[skip];
        if seq != expected_seq.raw() {
            continue;
        }
        let entity = remaining[skip + 1];
        let group = remaining[skip + 2];
        if matches!(
            RecordKind::from_entity_group(entity, group),
            RecordKind::Unknown { .. }
        ) {
            continue;
        }
        if buffer.skip(skip).is_err() {
            return false;
        }
        frame.resync_count += 1;
        return true;
    }
    false
}
