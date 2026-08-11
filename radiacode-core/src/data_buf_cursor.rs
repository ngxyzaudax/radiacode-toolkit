use radiacode_protocol::{DataBufFrame, DecodeWarningKind, Seq};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SeqGap {
    pub expected: u8,
    pub got: u8,
    pub lost: u8,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct DataBufCursor {
    last_seq: Option<Seq>,
    pub total_gaps: u64,
    pub total_lost: u64,
}

impl DataBufCursor {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn observe_frame(&mut self, frame: &DataBufFrame) -> Vec<SeqGap> {
        let mut gaps = decode_seq_gaps(frame);
        if let (Some(last), Some(first)) = (self.last_seq, frame_first_seq(frame)) {
            push_cross_poll_gap(&mut gaps, last, first);
        }
        if let Some(last) = frame_last_seq(frame) {
            self.last_seq = Some(last);
        }
        for gap in &gaps {
            self.total_gaps = self.total_gaps.saturating_add(1);
            self.total_lost = self.total_lost.saturating_add(u64::from(gap.lost));
        }
        gaps
    }
}

fn decode_seq_gaps(frame: &DataBufFrame) -> Vec<SeqGap> {
    frame
        .warnings
        .iter()
        .filter_map(|warning| match warning.kind {
            DecodeWarningKind::SeqJump { expected, got } => Some(SeqGap {
                expected,
                got,
                lost: Seq::new(got).lost_since(Seq::new(expected)),
            }),
            _ => None,
        })
        .collect()
}

fn push_cross_poll_gap(gaps: &mut Vec<SeqGap>, last: Seq, first: Seq) {
    let expected = last.next();
    if first == expected {
        return;
    }
    let candidate = SeqGap {
        expected: expected.raw(),
        got: first.raw(),
        lost: first.lost_since(expected),
    };
    if gaps
        .iter()
        .any(|gap| gap.expected == candidate.expected && gap.got == candidate.got)
    {
        return;
    }
    gaps.push(candidate);
}

fn frame_first_seq(frame: &DataBufFrame) -> Option<Seq> {
    frame.records.first().map(|record| record.header().seq)
}

fn frame_last_seq(frame: &DataBufFrame) -> Option<Seq> {
    frame.records.last().map(|record| record.header().seq)
}

#[cfg(test)]
mod tests {
    use radiacode_protocol::decode_data_buf;

    use super::*;

    fn build_realtime(seq: u8) -> Vec<u8> {
        let count_rate = 5.0f32.to_le_bytes();
        let dose_rate = 0.001f32.to_le_bytes();
        let mut bytes = vec![seq, 0, 0, 0, 0, 0, 0];
        bytes.extend_from_slice(&count_rate);
        bytes.extend_from_slice(&dose_rate);
        bytes.extend_from_slice(&[0u8; 7]);
        bytes
    }

    fn build_rare(seq: u8) -> Vec<u8> {
        vec![
            seq, 0, 3, 0, 0, 0, 0, 10, 0, 0, 0, 0, 0, 0, 0, 0xd4, 0x10, 0x34, 0x21, 0x00, 0x00,
        ]
    }

    #[test]
    fn ignores_non_rate_records_between_polls() {
        let mut cursor = DataBufCursor::default();
        let gaps1 = cursor.observe_frame(&decode_data_buf(&build_realtime(10)));
        assert!(gaps1.is_empty());

        let mut bytes = build_rare(11);
        bytes.extend_from_slice(&build_realtime(12));
        let gaps2 = cursor.observe_frame(&decode_data_buf(&bytes));
        assert!(gaps2.is_empty(), "unexpected gaps: {gaps2:?}");
    }

    #[test]
    fn reports_real_gap_between_polls() {
        let mut cursor = DataBufCursor::default();
        cursor.observe_frame(&decode_data_buf(&build_realtime(10)));
        let gaps = cursor.observe_frame(&decode_data_buf(&build_realtime(12)));
        assert_eq!(gaps.len(), 1);
        assert_eq!(gaps[0].expected, 11);
        assert_eq!(gaps[0].got, 12);
        assert_eq!(gaps[0].lost, 1);
    }
}
