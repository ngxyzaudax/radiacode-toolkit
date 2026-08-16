use radiacode_protocol::{DataBufFrame, DecodeWarningKind, Seq};

const MAX_PLAUSIBLE_GAP: u8 = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SeqGap {
    pub expected: u8,
    pub got: u8,
    pub lost: u32,
    pub reset: bool,
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
        if let (Some(last), Some(first)) = (self.last_seq, frame.first_header_seq) {
            push_cross_poll_gap(&mut gaps, last, first);
        }
        if let Some(last) = frame.last_header_seq {
            self.last_seq = Some(last);
        }
        for gap in &gaps {
            if gap.reset {
                continue;
            }
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
            DecodeWarningKind::SeqJump { expected, got } => {
                Some(seq_gap(Seq::new(got), Seq::new(expected)))
            }
            _ => None,
        })
        .collect()
}

fn seq_gap(got: Seq, expected: Seq) -> SeqGap {
    if got == expected {
        return SeqGap {
            expected: expected.raw(),
            got: got.raw(),
            lost: 0,
            reset: false,
        };
    }
    if !got.is_newer_than(expected) {
        return SeqGap {
            expected: expected.raw(),
            got: got.raw(),
            lost: 0,
            reset: true,
        };
    }
    let Some(lost) = got.lost_since(expected) else {
        return SeqGap {
            expected: expected.raw(),
            got: got.raw(),
            lost: 0,
            reset: true,
        };
    };
    if lost > u32::from(MAX_PLAUSIBLE_GAP) {
        return SeqGap {
            expected: expected.raw(),
            got: got.raw(),
            lost: 0,
            reset: true,
        };
    }
    SeqGap {
        expected: expected.raw(),
        got: got.raw(),
        lost,
        reset: false,
    }
}

fn push_cross_poll_gap(gaps: &mut Vec<SeqGap>, last: Seq, first: Seq) {
    let expected = last.next();
    if first == expected {
        return;
    }
    let candidate = seq_gap(first, expected);
    if gaps
        .iter()
        .any(|gap| gap.expected == candidate.expected && gap.got == candidate.got)
    {
        return;
    }
    gaps.push(candidate);
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
        assert!(!gaps[0].reset);
    }

    #[test]
    fn reset_after_cursor_clear_has_no_cross_poll_gap() {
        let mut cursor = DataBufCursor::default();
        cursor.observe_frame(&decode_data_buf(&build_realtime(200)));
        cursor.reset();
        let gaps = cursor.observe_frame(&decode_data_buf(&build_realtime(5)));
        assert!(gaps.is_empty(), "unexpected gaps after reset: {gaps:?}");
    }

    #[test]
    fn backwards_sequence_reports_reset_not_loss() {
        let mut cursor = DataBufCursor::default();
        cursor.observe_frame(&decode_data_buf(&build_realtime(200)));
        let gaps = cursor.observe_frame(&decode_data_buf(&build_realtime(5)));
        assert_eq!(gaps.len(), 1);
        assert!(gaps[0].reset);
        assert_eq!(gaps[0].lost, 0);
    }
}
