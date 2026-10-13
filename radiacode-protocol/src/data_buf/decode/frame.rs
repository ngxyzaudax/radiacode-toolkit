use std::ops::ControlFlow;

use crate::buffer::BytesBuffer;

use super::super::group::RecordKind;
use super::super::header::{DeviceTicks, RecordHeader};
use super::super::sanity::record_passes_sanity;
use super::super::seq::Seq;
use super::records::parse_payload;
use super::resync::{HEADER_LEN, resync_allowed, try_resync, unknown_record_tail};
use super::warnings::{DataBufFrame, DecodeWarningKind};

pub fn decode_data_buf(data: &[u8]) -> DataBufFrame {
    let mut decoder = FrameDecoder {
        data,
        buffer: BytesBuffer::new(data.to_vec()),
        frame: DataBufFrame::default(),
        expected_seq: None,
    };
    while decoder.buffer.size() >= HEADER_LEN {
        if decoder.step().is_break() {
            break;
        }
    }
    decoder.finish()
}

struct FrameDecoder<'a> {
    data: &'a [u8],
    buffer: BytesBuffer,
    frame: DataBufFrame,
    expected_seq: Option<Seq>,
}

impl FrameDecoder<'_> {
    fn offset(&self) -> usize {
        self.data.len().saturating_sub(self.buffer.size())
    }

    fn step(&mut self) -> ControlFlow<()> {
        let offset = self.offset();
        let Some(header) = parse_header(&mut self.buffer) else {
            return ControlFlow::Break(());
        };
        self.frame.note_header_seq(header.seq);
        if let RecordKind::Unknown { entity, group } = header.kind {
            return self.handle_unknown_record(offset, entity, group);
        }
        self.check_sequence(offset, header.seq);
        self.expected_seq = Some(header.seq.next());
        self.decode_record(offset, header)
    }

    fn handle_unknown_record(&mut self, offset: usize, entity: u8, group: u8) -> ControlFlow<()> {
        let tail = unknown_record_tail(self.data, self.buffer.size());
        self.frame.warn(
            offset,
            DecodeWarningKind::UnknownRecord {
                entity,
                group,
                tail,
            },
        );
        let Some(expected) = self.expected_seq else {
            return ControlFlow::Break(());
        };
        if resync_allowed(self.frame.resync_count) && try_resync(&mut self.buffer, expected) {
            self.frame.resync_count += 1;
            return ControlFlow::Continue(());
        }
        ControlFlow::Break(())
    }

    fn check_sequence(&mut self, offset: usize, seq: Seq) {
        if let Some(expected) = self.expected_seq
            && seq != expected
        {
            self.frame.warn(
                offset,
                DecodeWarningKind::SeqJump {
                    expected: expected.raw(),
                    got: seq.raw(),
                },
            );
        }
    }

    fn decode_record(&mut self, offset: usize, header: RecordHeader) -> ControlFlow<()> {
        match parse_payload(&mut self.buffer, header) {
            Ok(record) => {
                match record_passes_sanity(&record) {
                    Ok(()) => self.frame.records.push(record),
                    Err(failure) => self
                        .frame
                        .warn(offset, DecodeWarningKind::SanityRejected(failure)),
                }
                ControlFlow::Continue(())
            }
            Err(warning) => {
                self.frame.warnings.push(warning);
                ControlFlow::Break(())
            }
        }
    }

    fn finish(mut self) -> DataBufFrame {
        if self.buffer.size() > 0 && !self.frame.ended_on_unknown_record() {
            let offset = self.offset();
            self.frame.warn(offset, DecodeWarningKind::TruncatedTail);
        }
        self.frame
    }
}

fn parse_header(buffer: &mut BytesBuffer) -> Option<RecordHeader> {
    let seq = Seq::new(buffer.take_u8().ok()?);
    let entity = buffer.take_u8().ok()?;
    let group = buffer.take_u8().ok()?;
    let ts = DeviceTicks::new(buffer.take_i32_le().ok()?);
    let kind = RecordKind::from_entity_group(entity, group);
    Some(RecordHeader { seq, kind, ts })
}
