use super::super::records::DataBufRecord;
use super::super::sanity::SanityFailure;
use super::super::seq::Seq;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeWarningKind {
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
    pub first_header_seq: Option<Seq>,
    pub last_header_seq: Option<Seq>,
    pub resync_count: u32,
}

impl DataBufFrame {
    pub(super) fn warn(&mut self, offset: usize, kind: DecodeWarningKind) {
        self.warnings.push(DecodeWarning { offset, kind });
    }

    pub(super) fn note_header_seq(&mut self, seq: Seq) {
        if self.first_header_seq.is_none() {
            self.first_header_seq = Some(seq);
        }
        self.last_header_seq = Some(seq);
    }

    pub(super) fn ended_on_unknown_record(&self) -> bool {
        self.warnings
            .last()
            .is_some_and(|warning| matches!(warning.kind, DecodeWarningKind::UnknownRecord { .. }))
    }
}
